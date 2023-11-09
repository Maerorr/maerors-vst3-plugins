# All my VST3 plugins.

In this repository you can find all of my VST3 plugins written in Rust using the nih-plug framework.

## Building
After installing [Rust](https://rustup.rs/), you can compile the plugins as follows:

```shell
cd [plugin-directory-name]
cargo xtask bundle [plugin-name] --release
```

Current plugin names are as follows:
- biquad_filter
- chorus
- flanger
- disperser
- phaser

If i forget to update this file, the plugin name can be found in the `bundler.toml` file in each of the folders.

## Note
These plugins are not production-ready. They are written as a side-project to learn how DSP effects work. I test my plugins myself and there is always a chance that with certain parameter combinations the plugin will start outputting constant DC signal or very loud signal, because of an infinite feedback loop or simply a mistake in the code. If such a thing happens, open an issue and describe the problem and parameter values that cause problems.

## License
The MIT License (MIT)

Copyright (c) 2023 Hubert Łabuda

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
