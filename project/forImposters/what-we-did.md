- Looked at logs, noticed Plaintext_D0, Ciphertext_BU had an output of 1bit which was too small for size of 128bit
- used `stat` to check the last mtime (Modified time) for all the `.sv` files in srcs.
```
aes128.sv 2023-12-01 22:26:46.000000000 -0800
aes128Pkg.sv 2023-12-01 20:14:00.000000000 -0800
cipherRound.sv 2023-12-01 20:13:46.000000000 -0800
debouncer.sv 2023-12-01 19:34:16.000000000 -0800
flipflop.sv 2023-12-01 19:34:26.000000000 -0800
keyExpansion.sv 2023-12-01 22:26:42.000000000 -0800
micColumn.sv 2023-12-01 20:09:08.000000000 -0800
mixColumn.sv 2023-12-01 20:38:24.000000000 -0800
mixMatrix.sv 2023-12-01 20:11:56.000000000 -0800
sbox.sv 2023-12-01 20:11:44.000000000 -0800
subMatrix.sv 2023-12-01 20:11:28.000000000 -0800
subWord.sv 2023-12-01 20:10:52.000000000 -0800
```
- noticed `aes128.sv` and `keyExpansion.sv` was modified 2 hours after the other aes related files
- using header from aes files looked up and found the original module implementation
- using `diff` compared original files to provided files
```
aes128.sv
> output logic [127:0] Ciphertext_BU,
---
KeyExpansion.sv
> Ciphertext_BU = Cipherkey_DI;
```



