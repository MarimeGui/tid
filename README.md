# TID Library
A library I wrote to read TID texture / image files found in Hyperdimension games.

I reverse-engineered the format itself, and took some pieces of code to extract some of the textures.

Credits:
 - [Z-order curve](https://en.wikipedia.org/wiki/Z-order_curve) article on Wikipedia to understand Morton codes
 - [bcndecode](https://github.com/ifeherva/bcndecode) crate on Github for the BC1 decompression algorithm
 - [GXTConvert](https://github.com/xdanieldzd/GXTConvert) for a correct (and fast!) de-swizzling algorithm to use for these textures
 - [Scarlet](https://github.com/xdanieldzd/Scarlet) project for telling me that I needed to reorder the decompressed blocks of data, not the pixels
