# Improvements

- move all logic that involves reading/writing the binary format to the binfmt module
- have map_to_loci take some reader/iterator over map records and an iterator over source records
- have map_records take some writer and an iterator over source records
- create rsid, chrom and pos types and make invalid states unrepresentable
  - only the binfmt should have access to or use the "x_to_u#" functions
- abstract the record types and remove all "read_u8_at" behind an abstraction like "read_rsid_at" or "read_record_at"
- better error handling
- abstract the map over generic types so that maps can be created from any T1 -> T2 as long as they both implement w/e traits are required
- add num_records to END of file
- write_map_records should be a function in the binfmt module and it should handle file prepend/append for num records
  - the index module is responsible for parsing input TSV and calling the binfmt functions
- add docstrings
- clap
- do map sanity check?
