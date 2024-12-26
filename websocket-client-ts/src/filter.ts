export type Operator =
    | { GreaterThan: number }
    | { LessThan: number }
    | { GreaterOrEqual: number }
    | { LessOrEqual: number }
    | { Equals: any }
    | { NotEqual: any }
    | { StartsWith: string }
    | { EndsWith: string }
    | { Contains: string }
    | { ArrayContains: any }
    | { HasKey: string }
    | { And: Filter[] }
    | { Or: Filter[] };

export interface Filter {
    path: string;
    operator: Operator;
}
