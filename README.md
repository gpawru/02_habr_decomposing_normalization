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

| язык |  NFD  |      |  NFKD  |      |  NFD (dec) |  |
| ---- |  ---  |  --  |  ----  |  --  |  ---------- | - |
|      | **ICU4X** | **my** | **ICU4X** | **my** | **ICU4X** | **my** |
| arabic | 1211 | 586 | 1612 | 600 | 1251 | 558 |
| chinese | 870 | 250 | 1792 | 441 | 892 | 246 |
| czech | 889 | 668 | 1050 | 652 | 892 | 576 |
| dutch | 179 | 111 | 193 | 112 | 176 | 114 |
| english | 193 | 112 | 214 | 113 | 190 | 106 |
| french | 353 | 246 | 390 | 247 | 343 | 217 |
| german | 263 | 180 | 284 | 173 | 235 | 156 |
| greek | 1499 | 846 | 1894 | 830 | 1498 | 750 |
| hebrew | 1082 | 469 | 1447 | 454 | 1087 | 456 |
| hindi | 868 | 414 | 1164 | 412 | 871 | 416 |
| italian | 253 | 169 | 288 | 167 | 244 | 150 |
| japanese | 1069 | 394 | 1820 | 408 | 1085 | 345 |
| korean | 2670 | 1329 | 3255 | 1326 | 2144 | 707 |
| persian | 1135 | 498 | 1534 | 507 | 1124 | 480 |
| polish | 662 | 480 | 786 | 468 | 634 | 438 |
| portuguese | 294 | 196 | 314 | 193 | 276 | 171 |
| russian | 1118 | 496 | 1528 | 492 | 1131 | 483 |
| serbian | 1076 | 474 | 1464 | 473 | 1060 | 480 |
| spanish | 329 | 225 | 360 | 219 | 307 | 190 |
| thai | 900 | 474 | 1325 | 481 | 918 | 450 |
| turkish | 742 | 539 | 851 | 541 | 723 | 492 |
| ukrainian | 1124 | 537 | 1516 | 504 | 1133 | 483 |
| vietnamese | 1895 | 1332 | 2220 | 1406 | 1679 | 1139 |
