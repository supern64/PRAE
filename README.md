# Pyongyang Racer Asset Extractor (PRAE)
Rust tool for extracting `1.dat` and `common.dat` archives from Pyongyang Racer.
## Usage
Download executables from the releases section, or compile it yourself with `cargo build`
### Compressing an archive
```
prae zip <folder> [file]
prae zip 1 1.dat
```
### Decompressing an archive
```
prae unzip <file> [folder]
prae unzip common.dat common
```
### Listing content in archives
```
prae list <file>
prae list common.dat
```
More information on the file format here soon
