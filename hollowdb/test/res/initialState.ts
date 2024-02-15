export default {
  owner: "",
  verificationKeys: {
    auth: null,
  },
  isProofRequired: {
    auth: false,
  },
  canEvolve: false,
  whitelist: {
    put: {},
    update: {},
    set: {},
  },
  isWhitelistRequired: {
    put: false,
    update: false,
    set: false,
  },
} as const;
