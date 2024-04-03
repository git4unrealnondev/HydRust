Need to do a rewrite of this using shared info. 
Made this while I was learning rust so this is pretty poo code.
Going to ideally drop vec & tuples that get passed back and forth. 
Ideally I would use enum's as switches and structs as a sane way.
the no mangle parser function is pretty bad. Going to need to rework that :C

TLDR: Scraper code compiles down to a library that the main software can call dynamically. 
 - Probably not a secure way of doing this but idk it works without having to have the user
 - editing source code directly then that's a win in my books


