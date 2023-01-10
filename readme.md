# libsndfile

- Install libsndfile binaries https://github.com/bastibe/libsndfile-binaries
- Run autogen.sh
- mv the newly created libsndfile/src/.libs to ./libsndfile-src


# steps to run electron app with updated core

- run cargo build in core
- run package.json build-update in node
- run release/app/package.json postinstall
- Potentially update the release/app to  "permute-node": "../../../permute-node/permute-node-0.1.0.tgz"

