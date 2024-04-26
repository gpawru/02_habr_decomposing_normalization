# NF(K)D нормализация строк Unicode

примеры для статьи на Хабре: _вставить ссылку_

### структура репозитория:

- [**benches**](benches) - бенчмарки нормализации
- [**tests**](tests) - тесты нормализации
- **data** - "запечённые" данные декомпозиции
- [**decomposing**](decomposing) - нормализация строк
- **test_data** - данные для тестирования и бенчмарков

### подготовка данных:

- парсинг UCD: https://github.com/gpawru/unicode_data
- запекание данных: https://github.com/gpawru/unicode_bakery

### запуск тестов и бенчмарков:

```
make test
```

```
make bench
```
*(результат - в виде CSV)*

## Бенчмарки, µs

| язык       | NFD       |        | NFKD      |        | NFD (dec) |        |
| ---------- | --------- | ------ | --------- | ------ | --------- | ------ |
|            | **ICU4X** | **my** | **ICU4X** | **my** | **ICU4X** | **my** |
| arabic     | 1239      | 581    | 1620      | 594    | 1242      | 545    | 
| chinese    | 905       | 245    | 1634      | 426    | 880       | 240    | 
| czech      | 923       | 631    | 1006      | 640    | 870       | 539    | 
| dutch      | 182       | 115    | 195       | 113    | 177       | 114    | 
| english    | 193       | 111    | 215       | 111    | 188       | 108    | 
| french     | 357       | 237    | 386       | 229    | 340       | 210    | 
| german     | 252       | 172    | 290       | 171    | 244       | 154    | 
| greek      | 1478      | 812    | 1921      | 787    | 1511      | 733    | 
| hebrew     | 1091      | 461    | 1471      | 447    | 1103      | 465    | 
| hindi      | 862       | 405    | 1178      | 409    | 858       | 409    | 
| italian    | 260       | 170    | 298       | 166    | 244       | 151    | 
| japanese   | 1043      | 381    | 1699      | 399    | 1073      | 341    | 
| korean     | 2673      | 1263   | 3242      | 1238   | 2170      | 711    | 
| persian    | 1139      | 510    | 1527      | 489    | 1133      | 480    | 
| polish     | 667       | 477    | 772       | 475    | 640       | 431    | 
| portuguese | 290       | 186    | 310       | 182    | 271       | 163    | 
| russian    | 1109      | 494    | 1512      | 497    | 1113      | 472    | 
| serbian    | 1062      | 483    | 1458      | 461    | 1062      | 476    | 
| spanish    | 327       | 210    | 357       | 207    | 317       | 190    | 
| thai       | 914       | 431    | 1293      | 459    | 925       | 427    | 
| turkish    | 742       | 548    | 863       | 537    | 725       | 487    | 
| ukrainian  | 1131      | 508    | 1529      | 484    | 1194      | 482    | 
| vietnamese | 1927      | 1278   | 2206      | 1232   | 1677      | 1114   | 
