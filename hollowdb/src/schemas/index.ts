import { Static, Type } from "@sinclair/typebox";

export const Get = Type.Object({
  key: Type.String(),
});
export const GetMany = Type.Object({
  keys: Type.Array(Type.String()),
});

export const Put = Type.Object({
  key: Type.String(),
  value: Type.Any(),
});
export const PutMany = Type.Object({
  keys: Type.Array(Type.String()),
  values: Type.Array(Type.Any()),
});

export const Remove = Type.Object({
  key: Type.String(),
  proof: Type.Optional(Type.Any()),
});

export const Update = Type.Object({
  key: Type.String(),
  value: Type.Any(),
  proof: Type.Optional(Type.Any()),
});

export const Set = Type.Object({
  key: Type.String(),
  value: Type.Any(),
});
export const SetMany = Type.Object({
  keys: Type.Array(Type.String()),
  values: Type.Array(Type.Any()),
});

export const Clear = Type.Object({
  keys: Type.Optional(Type.Array(Type.String())),
});

export type Get = Static<typeof Get>;
export type GetMany = Static<typeof GetMany>;
export type Put = Static<typeof Put>;
export type PutMany = Static<typeof PutMany>;
export type Update = Static<typeof Update>;
export type Remove = Static<typeof Remove>;
export type Set = Static<typeof Set>;
export type SetMany = Static<typeof SetMany>;
export type Clear = Static<typeof Clear>;
