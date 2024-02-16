import { Static, Type } from "@sinclair/typebox";

export const Get = Type.Object({
  key: Type.String(),
});
export type Get = Static<typeof Get>;

export const GetMany = Type.Object({
  keys: Type.Array(Type.String()),
});
export type GetMany = Static<typeof GetMany>;

export const Put = Type.Object({
  key: Type.String(),
  value: Type.Any(),
});
export type Put = Static<typeof Put>;

export const PutMany = Type.Object({
  keys: Type.Array(Type.String()),
  values: Type.Array(Type.Any()),
});
export type PutMany = Static<typeof PutMany>;

export const Remove = Type.Object({
  key: Type.String(),
  proof: Type.Optional(Type.Any()),
});
export type Remove = Static<typeof Remove>;

export const Update = Type.Object({
  key: Type.String(),
  value: Type.Any(),
  proof: Type.Optional(Type.Any()),
});
export type Update = Static<typeof Update>;

export const Set = Type.Object({
  key: Type.String(),
  value: Type.Any(),
});
export type Set = Static<typeof Set>;

export const SetMany = Type.Object({
  keys: Type.Array(Type.String()),
  values: Type.Array(Type.Any()),
});
export type SetMany = Static<typeof SetMany>;

export const Clear = Type.Object({
  keys: Type.Optional(Type.Array(Type.String())),
});
export type Clear = Static<typeof Clear>;
