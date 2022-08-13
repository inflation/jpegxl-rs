# Changelog

<a name="0.7.0-alpha0"></a>
## 0.7.0-alpha0 (2022-08-13)

### Fixed

- 💚 Make it build on CI [[e4d0a3f](https://github.com/inflation/jpegxl-rs/commit/e4d0a3ffa423ed22408dcfd400a57bff1cf815af)]


<a name="0.7.0-alpha.0"></a>
## 0.7.0-alpha.0 (2022-08-13)

### Added

- ✅ Add more coverage tests [[e015a26](https://github.com/inflation/jpegxl-rs/commit/e015a2619020c45e471c7f4cb8c9ad868acf9c26)]

### Changed

- ♻️ Separate source to a separate crate [[2f837b0](https://github.com/inflation/jpegxl-rs/commit/2f837b08fe66820cd089653c14cdfbd9ec639eea)]
- ♻️ Make DecoderResult more convenient [[8bbe231](https://github.com/inflation/jpegxl-rs/commit/8bbe2312fc92570c766b22ebdaba4171fc7530d5)]
- ⬆️ Upgrade to 0.7rc [[46e9b0c](https://github.com/inflation/jpegxl-rs/commit/46e9b0c16b8e17d07f2203fec051c4d9b82a39a2)]
- ♻️ Move jpegxl-sys into workspace [[d7a8716](https://github.com/inflation/jpegxl-rs/commit/d7a8716bf2e8eea618474793fba2e171e3aa8248)]
- ♻️ Accept a mut ref to buffer in decode_internal [[6319836](https://github.com/inflation/jpegxl-rs/commit/6319836c679e7d90a79aaa2061de3bfd699cd94f)]
- 🏗️ Use system &#x60;libjxl&#x60; by default [[6756788](https://github.com/inflation/jpegxl-rs/commit/6756788b043c5a5ef1ea742cc33195aa2a1d1249)]
- ⬆️ Update deps and rust versions [[0cc189a](https://github.com/inflation/jpegxl-rs/commit/0cc189aae3034c1f3abe5ef28f70aba126552bcf)]

### Miscellaneous

-  Merge commit &#x27;b5ee294732fc402bf8d9a3182fe25208e50733d5&#x27; as &#x27;jpegxl-sys&#x27; [[fb60093](https://github.com/inflation/jpegxl-rs/commit/fb60093bf73336406804ce4c2f97721c7cfcfa87)]
-  Squashed &#x27;jpegxl-sys/&#x27; content from commit a6b5275 [[b5ee294](https://github.com/inflation/jpegxl-rs/commit/b5ee294732fc402bf8d9a3182fe25208e50733d5)]
- 📝 Use gitmoji-changelog [[d92f845](https://github.com/inflation/jpegxl-rs/commit/d92f845a12f249a8576357f501fbf47305a9fcb7)]
-  Merge pull request [#12](https://github.com/inflation/jpegxl-rs/issues/12) from idria/master [[d2d6f3b](https://github.com/inflation/jpegxl-rs/commit/d2d6f3b1ab38e850dbf0f303a880af7935a5b247)]
-  update image-rs library [[141db95](https://github.com/inflation/jpegxl-rs/commit/141db95c94714f731b349468149bd5ada3265d91)]
- 📝 Update docs to the latest address [[b7a47af](https://github.com/inflation/jpegxl-rs/commit/b7a47af465e934ecb80997d91d56b37ed25d537d)]


<a name="0.6.1"></a>
## 0.6.1 (2021-11-03)

### Changed

- ⬆️ Upgrade to v0.6.1 [[d8128c5](https://github.com/inflation/jpegxl-rs/commit/d8128c55cc04a790bba0880f2ad889ee7074a745)]

### Miscellaneous

- 📝 Add CHANGELOG.md [[ca68582](https://github.com/inflation/jpegxl-rs/commit/ca6858246ecda5147e8823cb9a825e0d235afb1d)]
-  (cargo-release) start next development iteration 0.6.1-alpha.0 [[5344059](https://github.com/inflation/jpegxl-rs/commit/53440595760ca9cabb100fea17127e5f55e07d37)]


<a name="0.6.0"></a>
## 0.6.0 (2021-10-13)

### Added

- ✨ Add luma only color encoding [[112b645](https://github.com/inflation/jpegxl-rs/commit/112b645f641a5ae4340226ab2722b763e237ba73)]
- ✨ Automatically figure out pixel types [[b309bd6](https://github.com/inflation/jpegxl-rs/commit/b309bd668ad1172089954f0b9c41a803625c17e6)]
- ✅ Continue improve code coverage [[d43b3c6](https://github.com/inflation/jpegxl-rs/commit/d43b3c6a17b571f0ff8b88a1d129ebf420e4f209)]
- ✅ Continue improve code coverage [[be94896](https://github.com/inflation/jpegxl-rs/commit/be948960a8136ac4898ba8ca32d8ba7162d7db1e)]
- ✅ Increase code coverage [[bd0d306](https://github.com/inflation/jpegxl-rs/commit/bd0d306fb35911869e8c5e0d9517f2d215bea131)]

### Changed

- ⬆️ Upgrade &#x60;libjxl&#x60; to v0.6 [[782603a](https://github.com/inflation/jpegxl-rs/commit/782603a73bdb8ee1dafdc6c0d6c36e665ea474f2)]
- 🎨 Remove Cell [[9dda100](https://github.com/inflation/jpegxl-rs/commit/9dda100a1f685779342f6b6c375c7f52e8845341)]
- 🎨 Remove unused setters [[9bc3b09](https://github.com/inflation/jpegxl-rs/commit/9bc3b0987b620a07a6045d88f0063be68595079b)]
- 🎨 Memory manager API consistency [[de0755c](https://github.com/inflation/jpegxl-rs/commit/de0755c33ee1c39f2d56f2c062ecf53865a6e653)]
- 🚨 Fix clippy [[ae3fbfa](https://github.com/inflation/jpegxl-rs/commit/ae3fbfa0b4a81815264494b7195eae159bb54a03)]
- 🎨 Use &#x60;derive_builder&#x60; [[6734234](https://github.com/inflation/jpegxl-rs/commit/6734234f9e7d274693f78df16f317ccefd08825e)]
- 🎨 Remove &#x60;unwrap()&#x60;s [[b2e914f](https://github.com/inflation/jpegxl-rs/commit/b2e914f54d87c1502ad1a8dda80c278e186c7f74)]
- 🎨 New way to integrate &#x60;image&#x60; crate [[b1d1715](https://github.com/inflation/jpegxl-rs/commit/b1d17157b30426b7713dcef13c93dc0e129855f2)]

### Fixed

- 💚 Fix memory leaks [[2011241](https://github.com/inflation/jpegxl-rs/commit/2011241db6ca092530db96d330c064ef21a34b64)]
- 🐛 Fix [#9](https://github.com/inflation/jpegxl-rs/issues/9) [[0b825fa](https://github.com/inflation/jpegxl-rs/commit/0b825fa9506470673c1dc19b9b26c175a61053da)]
- 🐛 Fix potential use-after-free [[01ee1c6](https://github.com/inflation/jpegxl-rs/commit/01ee1c655385f1f2c8767f5bd673f4be97cfdf70)]
- 💚 Fix CI [[e4a37df](https://github.com/inflation/jpegxl-rs/commit/e4a37df09411385d6e78a443f371ef38ef8b39d0)]

### Miscellaneous

-  feat: Remove size requirement for JPEG data [[b7f7ac1](https://github.com/inflation/jpegxl-rs/commit/b7f7ac11cad5fd8fba652ef28c1c7866698f062e)]
- 📝 Update docs [[5313c05](https://github.com/inflation/jpegxl-rs/commit/5313c050a1d01687b0321c25403a95746fdc2fb2)]
-  (cargo-release) start next development iteration 0.3.8-alpha.0 [[0f8d6d8](https://github.com/inflation/jpegxl-rs/commit/0f8d6d84c0e4ba5374bcfff5eb9bd5f6183acecd)]


<a name="0.6.2-alpha0"></a>
## 0.6.2-alpha0 (2022-07-06)

### Changed

- ♻️ Move jpegxl-sys into workspace [[79050a8](https://github.com/inflation/jpegxl-rs/commit/79050a8b0ea1a178f498e71909679d3989c2ca17)]
- ♻️ Accept a mut ref to buffer in decode_internal [[6319836](https://github.com/inflation/jpegxl-rs/commit/6319836c679e7d90a79aaa2061de3bfd699cd94f)]
- 🏗️ Use system &#x60;libjxl&#x60; by default [[6756788](https://github.com/inflation/jpegxl-rs/commit/6756788b043c5a5ef1ea742cc33195aa2a1d1249)]
- ⬆️ Update deps and rust versions [[0cc189a](https://github.com/inflation/jpegxl-rs/commit/0cc189aae3034c1f3abe5ef28f70aba126552bcf)]

### Miscellaneous

-  Merge commit &#x27;b5ee294732fc402bf8d9a3182fe25208e50733d5&#x27; as &#x27;jpegxl-sys&#x27; [[fb60093](https://github.com/inflation/jpegxl-rs/commit/fb60093bf73336406804ce4c2f97721c7cfcfa87)]
-  Squashed &#x27;jpegxl-sys/&#x27; content from commit a6b5275 [[b5ee294](https://github.com/inflation/jpegxl-rs/commit/b5ee294732fc402bf8d9a3182fe25208e50733d5)]
- 📝 Use gitmoji-changelog [[d92f845](https://github.com/inflation/jpegxl-rs/commit/d92f845a12f249a8576357f501fbf47305a9fcb7)]
-  Merge pull request [#12](https://github.com/inflation/jpegxl-rs/issues/12) from idria/master [[d2d6f3b](https://github.com/inflation/jpegxl-rs/commit/d2d6f3b1ab38e850dbf0f303a880af7935a5b247)]
-  update image-rs library [[141db95](https://github.com/inflation/jpegxl-rs/commit/141db95c94714f731b349468149bd5ada3265d91)]
- 📝 Update docs to the latest address [[b7a47af](https://github.com/inflation/jpegxl-rs/commit/b7a47af465e934ecb80997d91d56b37ed25d537d)]


<a name="0.3.7"></a>
## 0.3.7 (2021-04-13)

### Added

- 👷‍♂️ Add code coverage [[1d67f17](https://github.com/inflation/jpegxl-rs/commit/1d67f17660e5f636f3fa9a656308706f19413474)]

### Changed

- ⬆️ Bump to v0.3.7 [[cf0152e](https://github.com/inflation/jpegxl-rs/commit/cf0152ef03d74fc6668f912d22911e0b50a9bcc6)]

### Miscellaneous

- 📝 Update docs [[ecb3d5a](https://github.com/inflation/jpegxl-rs/commit/ecb3d5a25db14d2febd72d79544bf822e658d91a)]
-  (cargo-release) start next development iteration 0.3.6-alpha.0 [[f055d14](https://github.com/inflation/jpegxl-rs/commit/f055d14333f9ec763206e29ef0e0c5926435de91)]


<a name="0.3.5"></a>
## 0.3.5 (2021-03-25)

### Added

- ✨ Check signature first [[b781372](https://github.com/inflation/jpegxl-rs/commit/b78137215938a4fb13b651fd72db3c9cc7ddfaf5)]
- 👷‍♂️ Add security audit [[ab9efd8](https://github.com/inflation/jpegxl-rs/commit/ab9efd8d196849eb931796d6a4a25a9cbae10800)]

### Changed

- ⬆️ Upgrade to v0.3.5 [[8b5af3c](https://github.com/inflation/jpegxl-rs/commit/8b5af3caa48e784767164cb6bdae70958796a823)]
- 🎨 Break up decode [[8c8609b](https://github.com/inflation/jpegxl-rs/commit/8c8609b73bc2252564002c0ba3177d4d6e6893ba)]
- 🎨 Refactor to expose more functionalites [[9ea3da2](https://github.com/inflation/jpegxl-rs/commit/9ea3da2dd2fb91c5a0ed81859f057f8a2ddb71da)]
- 🎨 Use manually generated C bindings [[e386a33](https://github.com/inflation/jpegxl-rs/commit/e386a33b1aa86933d09cb5a343f20a670e729d4c)]

### Miscellaneous

-  (cargo-release) version 0.3.5 [[bafd61a](https://github.com/inflation/jpegxl-rs/commit/bafd61ad3f87d9743d00e34b48a0e5a4f43e8350)]
-  (cargo-release) start next development iteration 0.3.4-alpha.0 [[bbeba07](https://github.com/inflation/jpegxl-rs/commit/bbeba077efc976578d396913f2c1fb12f493975e)]


<a name="0.3.3"></a>
## 0.3.3 (2021-03-13)

### Added

- ✨ Store JPEG reconstuction metadata [[19e4135](https://github.com/inflation/jpegxl-rs/commit/19e4135dba9cc4a2d818d315f6249bfcb9350e22)]
- ✨ Output icc profile [[6d7a470](https://github.com/inflation/jpegxl-rs/commit/6d7a470d88bbb2f7411f4f44a60e42256b8207a4)]

### Changed

- 🎨 Rename feature &#x60;without-build&#x60; to &#x60;system-jpegxl&#x60; [[ab7fd2d](https://github.com/inflation/jpegxl-rs/commit/ab7fd2df4b266944dbd54ddab64eda8a136e4bfe)]
- ⬆️ Bump libjxl version [[fad37a4](https://github.com/inflation/jpegxl-rs/commit/fad37a41dd1b16233dbe9d8e63b3d384b4daa237)]

### Miscellaneous

-  (cargo-release) version 0.3.3 [[449610d](https://github.com/inflation/jpegxl-rs/commit/449610d2221d583f2ab6369e833458a0996d488e)]
-  Merge pull request [#3](https://github.com/inflation/jpegxl-rs/issues/3) from inflation/0.3.3 [[31770a6](https://github.com/inflation/jpegxl-rs/commit/31770a69977c27742b6918c4e5d6be20b616e7f0)]
-  Make clippy happy [[e2d9ee3](https://github.com/inflation/jpegxl-rs/commit/e2d9ee331df498e0650dfe77f7ecd240b910d430)]
-  Fix docs [[488c48e](https://github.com/inflation/jpegxl-rs/commit/488c48e96df12cd662690e56d06d767e1d5a6b9c)]
-  Refactor builders [[2e396d2](https://github.com/inflation/jpegxl-rs/commit/2e396d2e084a05da30fb523c5b2f624a731b4b0a)]
-  Move generic on JxlDecoder to decode() [[a22432d](https://github.com/inflation/jpegxl-rs/commit/a22432d6923e42f473a06dd33cf35787d4296c6d)]
-  Allow reuse of parallel runner and memory manager [[30a4912](https://github.com/inflation/jpegxl-rs/commit/30a49122ccc9a5452bc96da92ad736852a58fbf9)]
-  Remove rayon [[5a6e308](https://github.com/inflation/jpegxl-rs/commit/5a6e3081aed67bae9df38411103d7bac425aafdb)]
-  (cargo-release) start next development iteration 0.3.3-alpha.0 [[4b76775](https://github.com/inflation/jpegxl-rs/commit/4b76775e1cc882f66fa21f35ef7d33042668b7b6)]


<a name="0.3.2"></a>
## 0.3.2 (2021-02-16)

### Added

- ✨ Add color encoding option [[2ae4255](https://github.com/inflation/jpegxl-rs/commit/2ae425507ef1fdd5aa9c68bb80dfcaa93fda52cb)]

### Changed

- 🎨 Mask names from &#x60;jpegxl-sys&#x60; crate [[79357a2](https://github.com/inflation/jpegxl-rs/commit/79357a2e428d66b1620e5145d8eca82a9db0ebac)]

### Miscellaneous

-  (cargo-release) version 0.3.2 [[d9ae5dd](https://github.com/inflation/jpegxl-rs/commit/d9ae5ddcc02890e5d31839762a163c5f296190bc)]
-  Update jpeg-xl to 0.3.1 [[c45511d](https://github.com/inflation/jpegxl-rs/commit/c45511d764404c7f74eb7867cfef8366f6993635)]
-  Fix docs link [[6db8317](https://github.com/inflation/jpegxl-rs/commit/6db8317302ffa08fb19a687e2dfd9c2db1463637)]
-  (cargo-release) start next development iteration 0.3.1-alpha.0 [[63b7462](https://github.com/inflation/jpegxl-rs/commit/63b74629fa7e5c1436da3cdb77e84e1127e85e83)]


<a name="0.3.0"></a>
## 0.3.0 (2021-01-30)

### Miscellaneous

-  (cargo-release) version 0.3.0 [[36de340](https://github.com/inflation/jpegxl-rs/commit/36de3405bfacd534e3b919cc5273fed0f9e002f7)]
-  Merge pull request [#2](https://github.com/inflation/jpegxl-rs/issues/2) from inflation/v0.3 [[eddb887](https://github.com/inflation/jpegxl-rs/commit/eddb887a9626a42ca3061c2fb0bf3d10df0c9e3e)]
-  Add API to encode from raw JPEG data losslessly [[fd21b5d](https://github.com/inflation/jpegxl-rs/commit/fd21b5db87bebd145a09f0532d64dc153100dfdd)]
-  Update to v0.3 [[bb12089](https://github.com/inflation/jpegxl-rs/commit/bb1208934adc569dbfa556bfa2ab7b9d02b775b1)]
-  Fix cache and dependencies [[cd72cb7](https://github.com/inflation/jpegxl-rs/commit/cd72cb79aba684e578219cf6093ef755dfcfc434)]
-  Add cache to the CI [[ba7f3f9](https://github.com/inflation/jpegxl-rs/commit/ba7f3f958134a0b34e403c07fef44db8f3c8e9e0)]
-  Add badges [[8e9bc21](https://github.com/inflation/jpegxl-rs/commit/8e9bc21fbc243356b294df234cfa434990445173)]
-  Enable pedantic clippy lints [[f320165](https://github.com/inflation/jpegxl-rs/commit/f3201656194d599d518b3b9a036a6c386d4c1db4)]
-  Add CI [[6d22596](https://github.com/inflation/jpegxl-rs/commit/6d225961cd7e343afd1380ab3e55e638cc989f07)]
-  Create rust.yml [[427af0b](https://github.com/inflation/jpegxl-rs/commit/427af0b7fc5873071fdfa0411f14577bbebc0664)]
-  (cargo-release) start next development iteration 0.2.4-alpha.0 [[eb366cd](https://github.com/inflation/jpegxl-rs/commit/eb366cd5054dc2a74724a315ef14ab76437a7efe)]


<a name="0.2.3"></a>
## 0.2.3 (2021-01-15)

### Miscellaneous

-  Fix ImageDecoder impl [[eb93c9a](https://github.com/inflation/jpegxl-rs/commit/eb93c9a5b1884d0286fba6a38b8f4fa723ace339)]
-  (cargo-release) version 0.2.3 [[114c11b](https://github.com/inflation/jpegxl-rs/commit/114c11b0fd3ac0a793c691435e939b31f7bd9af6)]
-  Update dependency [[3c32d2f](https://github.com/inflation/jpegxl-rs/commit/3c32d2f227afbbdb993d60c717f4a8d17c868d89)]
-  Bring back errors [[889f999](https://github.com/inflation/jpegxl-rs/commit/889f9994dd20c586eaabf5963cfb51dd486fef38)]
-  Let compiler infering type for &#x60;as&#x60; conversion [[e131b0e](https://github.com/inflation/jpegxl-rs/commit/e131b0eb6c75ca962b179b482304167ecebbf36a)]
-  Fix docs.rs build [[cdf10fe](https://github.com/inflation/jpegxl-rs/commit/cdf10fe52960071e15e2aeee45ac53b66a903b60)]
-  Don&#x27;t initialize buffer [[0c29600](https://github.com/inflation/jpegxl-rs/commit/0c296003afc2976c06779c9a13d7f0b7a97c4605)]
-  Fix docs.rs build [[ba009bc](https://github.com/inflation/jpegxl-rs/commit/ba009bc0fe550e6b2d3a222b5001c9b826351cb5)]


<a name="0.2.2"></a>
## 0.2.2 (2021-01-10)

### Miscellaneous

-  Refactor parallel runner structure. [[5d44f72](https://github.com/inflation/jpegxl-rs/commit/5d44f726c0f1b677b592e87b34a6929b2f2c727c)]
-  Add a simple threadpool [[98ca998](https://github.com/inflation/jpegxl-rs/commit/98ca998f27b967504661655a340c8f2f4ef5d5fb)]
-  Use returned pixel format [[60e2334](https://github.com/inflation/jpegxl-rs/commit/60e23346956311dc4fce37682b0a8599494a935c)]
-  Add encoder building options [[1fd818c](https://github.com/inflation/jpegxl-rs/commit/1fd818c3f602af2907643e6492506c8932e2b551)]
-  Reorganized doc [[9c16406](https://github.com/inflation/jpegxl-rs/commit/9c16406b4feb8619940aaa2857866ad7254fcb82)]
-  Fix buffer size. [[6a5ea79](https://github.com/inflation/jpegxl-rs/commit/6a5ea79a4442f763e6ea49385e8d0fbe45385385)]
-  Add Encoder [[1733839](https://github.com/inflation/jpegxl-rs/commit/1733839ecccbf4d69201f993f4c76d4e1ae7b212)]
-  Add decoder builder for better API [[0714b28](https://github.com/inflation/jpegxl-rs/commit/0714b2893aeb22653b7f9fd7992c1c6e1cd6aeb9)]
-  Update JXLImage [[1251bc9](https://github.com/inflation/jpegxl-rs/commit/1251bc92a04bddded9ec09cff8b7b7e581187d00)]
-  Refactor decoder [[1e0aea0](https://github.com/inflation/jpegxl-rs/commit/1e0aea0be8e7e9c7c9c7b1e97f77caac1b7e0e3f)]
-  Use Threadpool by default [[8cdb3c4](https://github.com/inflation/jpegxl-rs/commit/8cdb3c49c96f33313db5cfa25d44bda72ee6ae2a)]
-  Add more &#x60;image&#x60; crate support [[92f5096](https://github.com/inflation/jpegxl-rs/commit/92f50961a54627435b58c1a46f8fe7460aa5530b)]
-  Refactor some memory manager code [[478fe0a](https://github.com/inflation/jpegxl-rs/commit/478fe0a53a35d5e8f4834207f2ab9c74976576fe)]
-  Add benchmarks [[6d6249c](https://github.com/inflation/jpegxl-rs/commit/6d6249cdc2dbd3a6e48ebde14c841e54c8c31cdb)]


<a name="0.1.4"></a>
## 0.1.4 (2020-08-25)

### Miscellaneous

-  Add memory manager [[a642b84](https://github.com/inflation/jpegxl-rs/commit/a642b8460db042bb1f1f339663d045818495c678)]


<a name="0.1.3"></a>
## 0.1.3 (2020-08-25)

### Miscellaneous

-  Fix yanked dependency [[3cdb850](https://github.com/inflation/jpegxl-rs/commit/3cdb850c62c58079633b4cd1ec1a7a78e3f0dbd6)]


<a name="0.1.2"></a>
## 0.1.2 (2020-08-19)

### Miscellaneous

-  Add multithreading runner [[62b81c6](https://github.com/inflation/jpegxl-rs/commit/62b81c617ddc674c3b139ce6a2f4f37d60af215e)]
-  Build on docs.rs [[8874e82](https://github.com/inflation/jpegxl-rs/commit/8874e8232ed5e0bd2bd0c2da3f780d945977819f)]
-  Add GPL-3.0 license and notices [[ff1c088](https://github.com/inflation/jpegxl-rs/commit/ff1c088767107d9ea35fa5aa56d27caa475d260b)]
-  Add docs. [[dc4093b](https://github.com/inflation/jpegxl-rs/commit/dc4093b9a3f5267ffc7e86c8ad3cbda8f5a4fd3a)]


<a name="0.1.0"></a>
## 0.1.0 (2020-08-16)

### Miscellaneous

-  Prepare for publish [[e29edfd](https://github.com/inflation/jpegxl-rs/commit/e29edfd16c00cd401121b553baac1f94dbd7c9e5)]
-  Init commit [[253f434](https://github.com/inflation/jpegxl-rs/commit/253f434b00ddbf90d308394268a9840d1290ed9e)]


