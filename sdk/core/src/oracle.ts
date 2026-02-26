import { PublicKey, Connection, TransactionSignature } from "@solana/web3.js";
import { Program, AnchorProvider, BN } from "@coral-xyz/anchor";

/** Oracle program ID */
const SSS_ORACLE_PROGRAM_ID = new PublicKey(
  "8kRVqx5JN2rSfn2haXBqgaLnQBrXzNSYj6PH9fKRk5bN"
);

const ORACLE_SEED = Buffer.from("oracle");

/** Oracle configuration state */
export interface OracleConfigData {
  authority: PublicKey;
  stablecoinMint: PublicKey;
  switchboardFeed: PublicKey;
  baseCurrency: string;
  quoteCurrency: string;
  maxStalenessSlots: BN;
  maxConfidenceInterval: BN;
  lastPrice: BN;
  lastPriceTimestamp: BN;
  lastPriceSlot: BN;
  enabled: boolean;
  bump: number;
}

/** Price data returned by getPrice */
export interface PriceData {
  price: BN;
  decimals: number;
  priceFloat: number;
  slot: BN;
  timestamp: BN;
  baseCurrency: string;
  quoteCurrency: string;
  isStale: boolean;
}

/** Parameters for mint with oracle */
export interface MintWithOracleParams {
  collateralAmount: BN;
  minTokensOut: BN;
  recipient: PublicKey;
  stablecoinMint: PublicKey;
  stablecoinConfig: PublicKey;
  roleConfig: PublicKey;
  recipientTokenAccount: PublicKey;
  sssTokenProgramId: PublicKey;
  tokenProgramId: PublicKey;
}

/**
 * Derives the OracleConfig PDA address.
 * Seeds: ["oracle", stablecoin_mint]
 */
export function deriveOracleConfig(
  stablecoinMint: PublicKey,
  programId: PublicKey = SSS_ORACLE_PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [ORACLE_SEED, stablecoinMint.toBuffer()],
    programId
  );
}

/**
 * Oracle module for SSS - provides price feed integration for non-USD stablecoins.
 *
 * Supports Switchboard V2 aggregator feeds for price data.
 * Enables minting stablecoins pegged to non-USD currencies (EUR, BRL, CPI-indexed)
 * by converting collateral value through the oracle price feed.
 */
export class OracleModule {
  private connection: Connection;
  private provider: AnchorProvider;
  private programId: PublicKey;

  constructor(
    provider: AnchorProvider,
    programId: PublicKey = SSS_ORACLE_PROGRAM_ID
  ) {
    this.provider = provider;
    this.connection = provider.connection;
    this.programId = programId;
  }

  /**
   * Get the oracle configuration for a stablecoin mint.
   */
  async getOracleConfig(
    stablecoinMint: PublicKey
  ): Promise<OracleConfigData | null> {
    const [oracleConfigPda] = deriveOracleConfig(
      stablecoinMint,
      this.programId
    );

    try {
      const accountInfo = await this.connection.getAccountInfo(oracleConfigPda);
      if (!accountInfo) return null;

      return this.parseOracleConfig(accountInfo.data);
    } catch {
      return null;
    }
  }

  /**
   * Get the current price from the oracle.
   * Returns parsed price data with staleness check.
   */
  async getPrice(stablecoinMint: PublicKey): Promise<PriceData> {
    const config = await this.getOracleConfig(stablecoinMint);
    if (!config) {
      throw new Error("Oracle not initialized for this stablecoin");
    }

    if (!config.enabled) {
      throw new Error("Oracle is disabled");
    }

    const currentSlot = await this.connection.getSlot();
    const slotsSinceUpdate = currentSlot - config.lastPriceSlot.toNumber();
    const isStale = slotsSinceUpdate > config.maxStalenessSlots.toNumber();

    const priceDecimals = 9;
    const priceFloat =
      config.lastPrice.toNumber() / Math.pow(10, priceDecimals);

    return {
      price: config.lastPrice,
      decimals: priceDecimals,
      priceFloat,
      slot: config.lastPriceSlot,
      timestamp: config.lastPriceTimestamp,
      baseCurrency: config.baseCurrency,
      quoteCurrency: config.quoteCurrency,
      isStale,
    };
  }

  /**
   * Calculate the expected tokens out for a given collateral amount.
   * Does not submit a transaction - just a preview.
   */
  async calculateTokensOut(
    stablecoinMint: PublicKey,
    collateralAmount: BN
  ): Promise<{ tokensOut: BN; price: BN; priceFloat: number }> {
    const priceData = await this.getPrice(stablecoinMint);
    if (priceData.isStale) {
      throw new Error("Oracle price is stale - refresh price first");
    }

    const priceDecimals = new BN(1_000_000_000); // 10^9
    const tokensOut = collateralAmount.mul(priceDecimals).div(priceData.price);

    return {
      tokensOut,
      price: priceData.price,
      priceFloat: priceData.priceFloat,
    };
  }

  /**
   * Build the initialize_oracle instruction data.
   * Returns serialized instruction data for use with Transaction.
   */
  buildInitializeOracleIx(params: {
    stablecoinMint: PublicKey;
    switchboardFeed: PublicKey;
    baseCurrency: string;
    quoteCurrency: string;
    maxStalenessSlots: BN;
    maxConfidenceInterval: BN;
  }): {
    accounts: Record<string, PublicKey>;
    oracleConfigPda: PublicKey;
  } {
    const [oracleConfigPda] = deriveOracleConfig(
      params.stablecoinMint,
      this.programId
    );

    return {
      accounts: {
        oracleConfig: oracleConfigPda,
        stablecoinMint: params.stablecoinMint,
        switchboardFeed: params.switchboardFeed,
        authority: this.provider.wallet.publicKey,
        systemProgram: new PublicKey(
          "11111111111111111111111111111111"
        ),
      },
      oracleConfigPda,
    };
  }

  /**
   * Build the get_price instruction accounts.
   */
  buildGetPriceIx(
    stablecoinMint: PublicKey,
    switchboardFeed: PublicKey
  ): {
    accounts: Record<string, PublicKey>;
  } {
    const [oracleConfigPda] = deriveOracleConfig(
      stablecoinMint,
      this.programId
    );

    return {
      accounts: {
        oracleConfig: oracleConfigPda,
        switchboardFeed,
      },
    };
  }

  /**
   * Build the mint_with_oracle instruction accounts.
   */
  buildMintWithOracleIx(params: MintWithOracleParams): {
    accounts: Record<string, PublicKey>;
  } {
    const [oracleConfigPda] = deriveOracleConfig(
      params.stablecoinMint,
      this.programId
    );

    return {
      accounts: {
        oracleConfig: oracleConfigPda,
        stablecoinConfig: params.stablecoinConfig,
        roleConfig: params.roleConfig,
        stablecoinMint: params.stablecoinMint,
        recipientTokenAccount: params.recipientTokenAccount,
        minter: this.provider.wallet.publicKey,
        sssTokenProgram: params.sssTokenProgramId,
        tokenProgram: params.tokenProgramId,
      },
    };
  }

  /**
   * Parse OracleConfig from raw account data.
   */
  private parseOracleConfig(data: Buffer): OracleConfigData {
    // Skip 8-byte Anchor discriminator
    let offset = 8;

    const authority = new PublicKey(data.subarray(offset, offset + 32));
    offset += 32;

    const stablecoinMint = new PublicKey(data.subarray(offset, offset + 32));
    offset += 32;

    const switchboardFeed = new PublicKey(data.subarray(offset, offset + 32));
    offset += 32;

    // Parse base_currency string
    const baseCurrencyLen = data.readUInt32LE(offset);
    offset += 4;
    const baseCurrency = data
      .subarray(offset, offset + baseCurrencyLen)
      .toString("utf8");
    offset += baseCurrencyLen;

    // Parse quote_currency string
    const quoteCurrencyLen = data.readUInt32LE(offset);
    offset += 4;
    const quoteCurrency = data
      .subarray(offset, offset + quoteCurrencyLen)
      .toString("utf8");
    offset += quoteCurrencyLen;

    const maxStalenessSlots = new BN(data.subarray(offset, offset + 8), "le");
    offset += 8;

    const maxConfidenceInterval = new BN(
      data.subarray(offset, offset + 8),
      "le"
    );
    offset += 8;

    const lastPrice = new BN(data.subarray(offset, offset + 8), "le");
    offset += 8;

    const lastPriceTimestamp = new BN(data.subarray(offset, offset + 8), "le");
    offset += 8;

    const lastPriceSlot = new BN(data.subarray(offset, offset + 8), "le");
    offset += 8;

    const enabled = data[offset] !== 0;
    offset += 1;

    const bump = data[offset];

    return {
      authority,
      stablecoinMint,
      switchboardFeed,
      baseCurrency,
      quoteCurrency,
      maxStalenessSlots,
      maxConfidenceInterval,
      lastPrice,
      lastPriceTimestamp,
      lastPriceSlot,
      enabled,
      bump,
    };
  }
}
