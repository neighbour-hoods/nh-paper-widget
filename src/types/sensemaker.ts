import { EntryHashB64 } from "@holochain-open-dev/core-types"

// what to do here? Implementing all the rust structs here will get
// rabbit-holey. Using `any` for now so that it works.
export interface SensemakerEntry {
  operator: any,
  operands: Array<any>,
  output_scheme: any,
  output_flat_value: any,
  start_gas: number
}

export interface SensemakerOutput {
  entry_hash: EntryHashB64,
  sensemaker_entry: SensemakerEntry
}
