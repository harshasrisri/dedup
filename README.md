# Dedup - Program to remove duplicate files

This program has born out of 2 goals :
-> Clean up my media collection like family photos, etc
-> Learn programming in Rust in the process.

```
dedup 0.1.0
Sriharsha Mucheli <harshasrisri@gmail.com>
Program to find duplicate files and take care of them

USAGE:
    dedup [FLAGS] [OPTIONS] --remote-list <remote_list>

FLAGS:
    -c, --commit-delete    Performs a dry run by default. Use this option to commit file deletions
    -d, --debug            Activate debug mode
    -h, --help             Prints help information
    -V, --version          Prints version information
    -v, --verbose          Verbose mode (-v, -vv, -vvv, etc.)

OPTIONS:
    -H, --hash-algo <hash_algo>        Type of Hashing algorigthm to use for checksumming. [default: Md5]
    -l, --local-path <local_path>      Local Path containing files that need to be checked for duplicates [default: .]
    -R, --remote-list <remote_list>    File containing list of remote files and hashes
```
