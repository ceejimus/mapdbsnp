# Improvements

- move all logic that involves reading/writing the binary format to the binfmt crate
- have map_to_loci take some reader/iterator over map records and an iterator over source records
- have map_records take some writer and an iterator over source records
- create rsid, chrom and pos types and make invalid states unrepresentable
- abstract the record types and remove all "read_u8_at" behind an abstraction like "read_rsid_at" or "read_record_at"
- better error handling
- abstract the map over generic types so that maps can be created from any T1 -> T2 as long as they both implement w/e traits are required
