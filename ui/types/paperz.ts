import { EntryHashB64, HeaderHashB64 } from "@holochain-open-dev/core-types"

export interface Paper {
  filename: string,
  blob_str: string
}

export interface PaperOutput {
  entry_hash: EntryHashB64,
  paper: Paper
}

export interface Annotation {
  paper_ref: EntryHashB64,
  page_num: number,
  paragraph_num: number,
  what_it_says: string,
  what_it_should_say: string,
}

export interface AnnotationOutput {
  entry_hash: EntryHashB64
  annotation: Annotation
}

