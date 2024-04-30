# How to add website support?

1. Add your source file to the [sources](https://github.com/NandeMD/robin/tree/main/robin_core/src/sources) folder, then add to [mod.rs](https://github.com/NandeMD/robin/blob/main/robin_core/src/sources/mod.rs). 

2. Inside your source file, create a struct. Your main struct for the website can contain any data you want but it must implement the `Serie` trait in [mod.rs](https://github.com/NandeMD/robin/blob/main/robin_core/src/sources/mod.rs) file.

3. Every serie struct's `download()` function must download all chapters to a temporary file (with tempfile crate) and return the temporary file's handle. This handle will be dropped automatically and tempfile will be deleted.

4. You don't have tou use `Chapter` trait at all. It is there for only convenience.

5. When you done with all of your trait implementations, add your website's definitive url to the [mathcher](https://github.com/NandeMD/robin/blob/main/robin_core/src/matcher.rs) function. All if/else if branches must return a `Ok(impl Serie)`. 

6. Add your site to [SITES.md](https://github.com/NandeMD/robin/blob/main/SITES.md) After that, you are done. Your serie is added to robin. Have a good time scraping.

## Example:
Although it may not be very well-written code, you can directly look into the [example](https://github.com/NandeMD/robin/blob/main/robin_core/src/sources/shijie_turkish.rs) file and even copy the parts that are useful to you.