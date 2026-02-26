/**
 * Error codes matching the on-chain StablecoinError enum.
 * Anchor error codes start at 6000 (0x1770) by default.
 */
export enum StablecoinErrorCode {
  Paused = 6000,
  Unauthorized = 6001,
  MinterQuotaExceeded = 6002,
  MinterNotFound = 6003,
  MaxMintersReached = 6004,
  MaxBurnersReached = 6005,
  ComplianceNotEnabled = 6006,
  AlreadyBlacklisted = 6007,
  NotBlacklisted = 6008,
  NameTooLong = 6009,
  SymbolTooLong = 6010,
  UriTooLong = 6011,
  ReasonTooLong = 6012,
  MathOverflow = 6013,
  AccountFrozen = 6014,
  InvalidAuthority = 6015,
  MinterAlreadyExists = 6016,
  BurnerAlreadyExists = 6017,
  BurnerNotFound = 6018,
}

/** Human-readable error messages keyed by error code */
export const ERROR_MESSAGES: Record<number, string> = {
  [StablecoinErrorCode.Paused]: "The stablecoin is currently paused",
  [StablecoinErrorCode.Unauthorized]:
    "Unauthorized: caller does not have the required role",
  [StablecoinErrorCode.MinterQuotaExceeded]: "Minter quota exceeded",
  [StablecoinErrorCode.MinterNotFound]: "Minter not found",
  [StablecoinErrorCode.MaxMintersReached]:
    "Maximum number of minters reached",
  [StablecoinErrorCode.MaxBurnersReached]:
    "Maximum number of burners reached",
  [StablecoinErrorCode.ComplianceNotEnabled]:
    "Compliance module is not enabled for this stablecoin",
  [StablecoinErrorCode.AlreadyBlacklisted]: "Address is already blacklisted",
  [StablecoinErrorCode.NotBlacklisted]: "Address is not blacklisted",
  [StablecoinErrorCode.NameTooLong]: "Name exceeds maximum length",
  [StablecoinErrorCode.SymbolTooLong]: "Symbol exceeds maximum length",
  [StablecoinErrorCode.UriTooLong]: "URI exceeds maximum length",
  [StablecoinErrorCode.ReasonTooLong]: "Reason exceeds maximum length",
  [StablecoinErrorCode.MathOverflow]: "Math overflow",
  [StablecoinErrorCode.AccountFrozen]: "Account is frozen",
  [StablecoinErrorCode.InvalidAuthority]: "Invalid authority",
  [StablecoinErrorCode.MinterAlreadyExists]: "Minter already exists",
  [StablecoinErrorCode.BurnerAlreadyExists]: "Burner already exists",
  [StablecoinErrorCode.BurnerNotFound]: "Burner not found",
};

/**
 * Parses an Anchor program error and returns a human-readable message.
 * Falls back to the original error message if the code is not recognized.
 */
export function parseStablecoinError(error: any): string {
  const code = error?.error?.errorCode?.number ?? error?.code;
  if (code !== undefined && ERROR_MESSAGES[code]) {
    return ERROR_MESSAGES[code];
  }
  return error?.message ?? "Unknown error";
}
