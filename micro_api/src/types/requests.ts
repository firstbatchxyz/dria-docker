/**
 * Route types are provided in the body of a request.
 *
 * If a value other than these is given as route, user gets a bad request.
 */
export type Route =
  | "GET"
  | "PUT"
  | "UPDATE"
  | "REMOVE"
  | "STATE"
  | "GET_MANY"
  | "PUT_MANY"
  | "REFRESH"
  | "GET_MANY_RAW";

type GetRequest = {
  route: "GET";
  data: {
    key: string;
  };
};

type GetRawRequest = {
  route: "GET_RAW";
  data: {
    key: string;
  };
};

type GetManyRequest = {
  route: "GET_MANY";
  data: {
    keys: string[];
  };
};

type GetManyRawRequest = {
  route: "GET_MANY_RAW";
  data: {
    keys: string[];
  };
};

type PutRequest<V> = {
  route: "PUT";
  data: {
    key: string;
    value: V;
  };
};

type PutManyRequest<V> = {
  route: "PUT_MANY";
  data: {
    keys: string[];
    values: V[];
  };
};

type UpdateRequest<V> = {
  route: "UPDATE";
  data: {
    key: string;
    value: V;
    proof: object;
  };
};

type RemoveRequest = {
  route: "REMOVE";
  data: {
    key: string;
    proof: object;
  };
};

type StateRequest = {
  route: "STATE";
  data: {};
};

type RefreshRequest = {
  route: "REFRESH";
  data: {};
};

type ClearRequest = {
  route: "CLEAR";
  data: {
    keys?: string[];
  };
};

export type Request<V> =
  | GetRequest
  | GetRawRequest
  | GetManyRequest
  | GetManyRawRequest
  | PutRequest<V>
  | PutManyRequest<V>
  | UpdateRequest<V>
  | RemoveRequest
  | StateRequest
  | RefreshRequest
  | ClearRequest;
