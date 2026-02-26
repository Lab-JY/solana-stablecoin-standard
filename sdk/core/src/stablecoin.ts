import {
  Connection,
  PublicKey,
  Keypair,
  TransactionSignature,
  SystemProgram,
} from "@solana/web3.js";
import {
  TOKEN_2022_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  getAccount,
} from "@solana/spl-token";
import { Program, AnchorProvider, Idl, BN } from "@coral-xyz/anchor";
import {
  StablecoinConfig,
  RoleConfig,
  InitializeParams,
  MintParams,
  BurnParams,
  HolderInfo,
  UpdateMinterParams,
  UpdateRolesParams,
  MinterAction,
} from "./types";
import {
  deriveStablecoinConfig,
  deriveRoleConfig,
} from "./pda";
import { ComplianceModule } from "./compliance";

/** Default program IDs */
const SSS_TOKEN_PROGRAM_ID = new PublicKey(
  "AhZamuppxULmpM9QGXcZJ9ZR3fvQbDd4gPsxLtDoMQmE"
);
const SSS_TRANSFER_HOOK_PROGRAM_ID = new PublicKey(
  "Gf5xP5YMRdhb7jRGiDsZW2guwwRMi4RQt4b5r44VPhTU"
);

/**
 * SolanaStablecoin is the main entry point for interacting with an SSS stablecoin.
 *
 * Use `SolanaStablecoin.create()` to deploy a new stablecoin, or
 * `SolanaStablecoin.load()` to connect to an existing one.
 */
export class SolanaStablecoin {
  private _config: StablecoinConfig | null = null;
  private _roleConfig: RoleConfig | null = null;
  private _complianceModule: ComplianceModule | null = null;

  private constructor(
    public readonly connection: Connection,
    public readonly programId: PublicKey,
    public readonly transferHookProgramId: PublicKey,
    public readonly mintAddress: PublicKey,
    public readonly configAddress: PublicKey,
    public readonly configBump: number,
    public readonly roleConfigAddress: PublicKey,
    public readonly roleConfigBump: number,
    private readonly program: Program
  ) {}

  /**
   * Creates and deploys a new stablecoin on-chain.
   * Sends the initialize transaction with the given parameters.
   *
   * @returns A new SolanaStablecoin instance connected to the deployed stablecoin.
   */
  static async create(
    provider: AnchorProvider,
    idl: Idl,
    params: InitializeParams,
    mintKeypair: Keypair,
    programId: PublicKey = SSS_TOKEN_PROGRAM_ID,
    transferHookProgramId: PublicKey = SSS_TRANSFER_HOOK_PROGRAM_ID
  ): Promise<{ stablecoin: SolanaStablecoin; signature: TransactionSignature }> {
    const programIdl = { ...idl, address: programId.toBase58() } as Idl;
    const program = new Program(programIdl, provider);
    const mint = mintKeypair.publicKey;

    const [configAddress, configBump] = deriveStablecoinConfig(programId, mint);
    const [roleConfigAddress, roleConfigBump] = deriveRoleConfig(
      programId,
      configAddress
    );

    const accounts: Record<string, PublicKey> = {
      authority: provider.wallet.publicKey,
      mint: mint,
      stablecoinConfig: configAddress,
      roleConfig: roleConfigAddress,
      transferHookProgram: SystemProgram.programId,
      tokenProgram: TOKEN_2022_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    };

    if (params.enableTransferHook) {
      accounts.transferHookProgram = transferHookProgramId;
    }

    const signature = await program.methods
      .initialize({
        name: params.name,
        symbol: params.symbol,
        uri: params.uri,
        decimals: params.decimals,
        enablePermanentDelegate: params.enablePermanentDelegate,
        enableTransferHook: params.enableTransferHook,
        defaultAccountFrozen: params.defaultAccountFrozen,
      })
      .accounts(accounts)
      .signers([mintKeypair])
      .rpc();

    const stablecoin = new SolanaStablecoin(
      provider.connection,
      programId,
      transferHookProgramId,
      mint,
      configAddress,
      configBump,
      roleConfigAddress,
      roleConfigBump,
      program
    );

    return { stablecoin, signature };
  }

  /**
   * Loads an existing stablecoin from chain by its mint address.
   */
  static async load(
    provider: AnchorProvider,
    idl: Idl,
    mint: PublicKey,
    programId: PublicKey = SSS_TOKEN_PROGRAM_ID,
    transferHookProgramId: PublicKey = SSS_TRANSFER_HOOK_PROGRAM_ID
  ): Promise<SolanaStablecoin> {
    const programIdl = { ...idl, address: programId.toBase58() } as Idl;
    const program = new Program(programIdl, provider);

    const [configAddress, configBump] = deriveStablecoinConfig(programId, mint);
    const [roleConfigAddress, roleConfigBump] = deriveRoleConfig(
      programId,
      configAddress
    );

    const stablecoin = new SolanaStablecoin(
      provider.connection,
      programId,
      transferHookProgramId,
      mint,
      configAddress,
      configBump,
      roleConfigAddress,
      roleConfigBump,
      program
    );

    // Verify the stablecoin exists by loading config
    await stablecoin.getConfig();

    return stablecoin;
  }

  /**
   * Refreshes the cached config and role config from chain.
   */
  async refresh(): Promise<void> {
    this._config = null;
    this._roleConfig = null;
    await Promise.all([this.getConfig(), this.getRoleConfig()]);
  }

  /**
   * Returns the cached StablecoinConfig, or fetches it if not cached.
   */
  get cachedConfig(): StablecoinConfig | null {
    return this._config;
  }

  /**
   * Returns the cached RoleConfig, or fetches it if not cached.
   */
  get cachedRoleConfig(): RoleConfig | null {
    return this._roleConfig;
  }

  /**
   * Mints tokens to a recipient. Caller must be a registered minter.
   */
  async mint(
    params: MintParams,
    minter: Keypair
  ): Promise<TransactionSignature> {
    const recipientAta = getAssociatedTokenAddressSync(
      this.mintAddress,
      params.recipient,
      false,
      TOKEN_2022_PROGRAM_ID
    );

    return this.program.methods
      .mintTokens(params.amount)
      .accountsStrict({
        minter: minter.publicKey,
        stablecoinConfig: this.configAddress,
        roleConfig: this.roleConfigAddress,
        mint: this.mintAddress,
        recipientTokenAccount: recipientAta,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([minter])
      .rpc();
  }

  /**
   * Burns tokens from the given token account. Caller must be a registered burner.
   */
  async burn(
    params: BurnParams,
    burner: Keypair
  ): Promise<TransactionSignature> {
    return this.program.methods
      .burnTokens(params.amount)
      .accountsStrict({
        burner: burner.publicKey,
        stablecoinConfig: this.configAddress,
        roleConfig: this.roleConfigAddress,
        mint: this.mintAddress,
        burnerTokenAccount: params.tokenAccount,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([burner])
      .rpc();
  }

  /**
   * Freezes a token account. Caller must be authority or pauser.
   */
  async freezeAccount(
    address: PublicKey,
    authority: Keypair
  ): Promise<TransactionSignature> {
    const tokenAccount = getAssociatedTokenAddressSync(
      this.mintAddress,
      address,
      false,
      TOKEN_2022_PROGRAM_ID
    );

    return this.program.methods
      .freezeAccount()
      .accountsStrict({
        authority: authority.publicKey,
        stablecoinConfig: this.configAddress,
        roleConfig: this.roleConfigAddress,
        mint: this.mintAddress,
        tokenAccount: tokenAccount,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([authority])
      .rpc();
  }

  /**
   * Thaws a frozen token account. Caller must be authority or pauser.
   */
  async thawAccount(
    address: PublicKey,
    authority: Keypair
  ): Promise<TransactionSignature> {
    const tokenAccount = getAssociatedTokenAddressSync(
      this.mintAddress,
      address,
      false,
      TOKEN_2022_PROGRAM_ID
    );

    return this.program.methods
      .thawAccount()
      .accountsStrict({
        authority: authority.publicKey,
        stablecoinConfig: this.configAddress,
        roleConfig: this.roleConfigAddress,
        mint: this.mintAddress,
        tokenAccount: tokenAccount,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([authority])
      .rpc();
  }

  /**
   * Pauses all stablecoin operations globally. Caller must be pauser.
   */
  async pause(pauser: Keypair): Promise<TransactionSignature> {
    return this.program.methods
      .pause()
      .accountsStrict({
        pauser: pauser.publicKey,
        stablecoinConfig: this.configAddress,
        roleConfig: this.roleConfigAddress,
      })
      .signers([pauser])
      .rpc();
  }

  /**
   * Unpauses the stablecoin. Caller must be pauser.
   */
  async unpause(pauser: Keypair): Promise<TransactionSignature> {
    return this.program.methods
      .unpause()
      .accountsStrict({
        pauser: pauser.publicKey,
        stablecoinConfig: this.configAddress,
        roleConfig: this.roleConfigAddress,
      })
      .signers([pauser])
      .rpc();
  }

  /**
   * Updates minter configuration (add/remove/update quota).
   * Caller must be the master authority.
   */
  async updateMinter(
    params: UpdateMinterParams,
    authority: Keypair
  ): Promise<TransactionSignature> {
    let actionArg: any;
    switch (params.action) {
      case MinterAction.Add:
        actionArg = {
          add: { address: params.address, quota: params.quota ?? new BN(0) },
        };
        break;
      case MinterAction.Remove:
        actionArg = { remove: { address: params.address } };
        break;
      case MinterAction.UpdateQuota:
        actionArg = {
          updateQuota: {
            address: params.address,
            newQuota: params.quota ?? new BN(0),
          },
        };
        break;
    }

    return this.program.methods
      .updateMinter(actionArg)
      .accountsStrict({
        authority: authority.publicKey,
        stablecoinConfig: this.configAddress,
        roleConfig: this.roleConfigAddress,
      })
      .signers([authority])
      .rpc();
  }

  /**
   * Updates role assignments. Caller must be the master authority.
   */
  async updateRoles(
    params: UpdateRolesParams,
    authority: Keypair
  ): Promise<TransactionSignature> {
    let actionArg: any;
    switch (params.role) {
      case "pauser":
        actionArg = { setPauser: { address: params.address } };
        break;
      case "blacklister":
        actionArg = { setBlacklister: { address: params.address } };
        break;
      case "seizer":
        actionArg = { setSeizer: { address: params.address } };
        break;
      default:
        throw new Error(`Unsupported role type: ${params.role}`);
    }

    return this.program.methods
      .updateRoles(actionArg)
      .accountsStrict({
        authority: authority.publicKey,
        stablecoinConfig: this.configAddress,
        roleConfig: this.roleConfigAddress,
      })
      .signers([authority])
      .rpc();
  }

  /**
   * Transfers master authority to a new address.
   * Caller must be the current master authority.
   */
  async transferAuthority(
    newAuthority: PublicKey,
    currentAuthority: Keypair
  ): Promise<TransactionSignature> {
    return this.program.methods
      .transferAuthority(newAuthority)
      .accountsStrict({
        authority: currentAuthority.publicKey,
        stablecoinConfig: this.configAddress,
        roleConfig: this.roleConfigAddress,
      })
      .signers([currentAuthority])
      .rpc();
  }

  /**
   * Returns the total circulating supply (totalMinted - totalBurned).
   */
  async getTotalSupply(): Promise<BN> {
    const config = await this.getConfig();
    return config.totalMinted.sub(config.totalBurned);
  }

  /**
   * Fetches and caches the on-chain StablecoinConfig.
   */
  async getConfig(): Promise<StablecoinConfig> {
    const account = await this.program.account.stablecoinConfig.fetch(
      this.configAddress
    );
    this._config = {
      authority: account.authority as PublicKey,
      mint: account.mint as PublicKey,
      name: account.name as string,
      symbol: account.symbol as string,
      uri: account.uri as string,
      decimals: account.decimals as number,
      paused: account.paused as boolean,
      totalMinted: account.totalMinted as BN,
      totalBurned: account.totalBurned as BN,
      enablePermanentDelegate: account.enablePermanentDelegate as boolean,
      enableTransferHook: account.enableTransferHook as boolean,
      defaultAccountFrozen: account.defaultAccountFrozen as boolean,
      transferHookProgram: (account.transferHookProgram as PublicKey) ?? null,
      bump: account.bump as number,
    };
    return this._config;
  }

  /**
   * Fetches and caches the on-chain RoleConfig.
   */
  async getRoleConfig(): Promise<RoleConfig> {
    const account = await this.program.account.roleConfig.fetch(
      this.roleConfigAddress
    );
    this._roleConfig = {
      stablecoin: account.stablecoin as PublicKey,
      masterAuthority: account.masterAuthority as PublicKey,
      pauser: account.pauser as PublicKey,
      minters: (account.minters as any[]).map((m) => ({
        address: m.address as PublicKey,
        quota: m.quota as BN,
        minted: m.minted as BN,
      })),
      burners: account.burners as PublicKey[],
      blacklister: account.blacklister as PublicKey,
      seizer: account.seizer as PublicKey,
      bump: account.bump as number,
    };
    return this._roleConfig;
  }

  /**
   * Returns all token holders for this stablecoin.
   * Uses getProgramAccounts to find all token accounts for this mint.
   */
  async getHolders(): Promise<HolderInfo[]> {
    const accounts = await this.connection.getProgramAccounts(
      TOKEN_2022_PROGRAM_ID,
      {
        filters: [
          { dataSize: 165 },
          {
            memcmp: {
              offset: 0,
              bytes: this.mintAddress.toBase58(),
            },
          },
        ],
      }
    );

    return accounts.map((account) => {
      const data = account.account.data;
      const owner = new PublicKey(data.slice(32, 64));
      const amount = new BN(data.slice(64, 72), "le");
      const state = data[108];

      return {
        address: owner,
        tokenAccount: account.pubkey,
        balance: amount,
        isFrozen: state === 2,
      };
    });
  }

  /**
   * Returns the ComplianceModule for SSS-2 operations.
   * Operations will fail on-chain with ComplianceNotEnabled for SSS-1 stablecoins.
   */
  get compliance(): ComplianceModule {
    if (!this._complianceModule) {
      this._complianceModule = new ComplianceModule(
        this.connection,
        this.program,
        this.programId,
        this.transferHookProgramId,
        this.mintAddress,
        this.configAddress,
        this.configBump,
        this.roleConfigAddress
      );
    }
    return this._complianceModule;
  }
}
