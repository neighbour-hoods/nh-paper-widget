import { EntryHashB64, HeaderHashB64 } from "@holochain-open-dev/core-types"

// Philosophical NH question: How much of a sensemaker entry does a 
// widget need to know about?

// Implementing all the rust structs here will get
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


export interface HolochainOutput {
  entry_hash: EntryHashB64,
  header_hash: HeaderHashB64
}

export interface StateMachineInput {
  path: string,
  expr: string
}

export interface StepStateMachineInput {
  path: string,
  entry_hash: EntryHashB64,
  action: string
}