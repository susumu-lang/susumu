# Susumu Standard Library

The Susumu Standard Library provides comprehensive, specialized function libraries that complement the auto-available core functions. This hybrid architecture ensures maximum productivity for common operations while providing enterprise-grade functionality for advanced use cases.

## Library Architecture

### Core Auto-Available Functions (43 functions)
These functions are immediately available without imports:

- **Math**: `add`, `subtract`, `multiply`, `divide` (4)
- **I/O**: `print`, `println`, `readFile`, `writeFile`, `appendFile`, `fileExists`, `fileInfo`, `listDir` (8)  
- **Conversions**: `toString`, `toNumber` (2)
- **Validation**: `isNull`, `isEmpty`, `isNumber`, `isString` (4)
- **Utilities**: `length`, `type`, `equals` (3)
- **JSON**: `parseJSON`, `toJSON` (2)
- **Arrays**: `filter`, `map`, `reduce` (3) 
- **Date/Time**: `now`, `nowMillis`, `formatDate`, `parseDate`, `addTime`, `timeDiff` (6)
- **HTTP**: `httpGet`, `httpPost`, `httpRequest` (3)
- **Parallel**: `httpGetParallel`, `httpPostParallel`, `readFilesParallel`, `mapParallel` (4)

### Standard Library Modules (200+ functions)
Specialized modules imported on-demand:

## Module Reference

### 📐 Math Module (`math.susu`)
**Import**: `math_module -> from <- import <- (sin, cos, factorial, isPrime)`

Advanced mathematical operations:
- **Trigonometry**: `sin`, `cos`, `tan` 
- **Number Theory**: `factorial`, `gcd`, `lcm`, `isPrime`, `fibonacci`
- **Statistics**: `mean`, `median`, `standardDeviation`
- **Geometry**: `distance`, `circleArea`, `sphereVolume`
- **Linear Algebra**: `matrixMultiply`, `matrixTranspose`

**Total**: 17 functions

### 🔤 String Module (`string.susu`)
**Import**: `string_module -> from <- import <- (split, join, regex, template)`

Comprehensive string processing:
- **Manipulation**: `split`, `join`, `trim`, `trimLeft`, `trimRight`
- **Case Conversion**: `toTitleCase`, `toCamelCase`, `toSnakeCase`, `toKebabCase`
- **Pattern Matching**: `contains`, `startsWith`, `endsWith`, `matches`
- **Analysis**: `countOccurrences`, `levenshteinDistance`, `wordCount`
- **Formatting**: `template`, `format`, `pad`, `padLeft`, `padRight`
- **Encoding**: `base64Encode`, `base64Decode`, `urlEncode`, `urlDecode`, `htmlEscape`
- **AI/NLP**: `sentiment`, `extractKeywords`, `summarize`

**Total**: 35 functions

### 📋 Array Module (`array.susu`)
**Import**: `array_module -> from <- import <- (chunk, flatten, unique, groupBy)`

Advanced array and collection operations:
- **Restructuring**: `chunk`, `flatten`, `zip`, `unzip`, `transpose`
- **Set Operations**: `unique`, `intersection`, `union`, `difference`
- **Grouping**: `groupBy`, `partition`
- **Sorting/Searching**: `binarySearch`, `quickSort`, `mergeSort`, `sortBy`
- **Statistics**: `min`, `max`, `sum`, `average`, `median`, `mode`
- **Generation**: `range`, `repeat`, `zeros`, `ones`
- **Functional**: `take`, `drop`, `takeWhile`, `dropWhile`, `scan`
- **Parallel**: `mapParallel`, `filterParallel`, `reduceParallel`

**Total**: 42 functions

### 📁 File System Module (`fs.susu`)
**Import**: `fs_module -> from <- import <- (watch, backup, compress, search)`

Enterprise file system operations:
- **File Operations**: `copy`, `move`, `createDirectory`, `removeDirectory`
- **Monitoring**: `watch`, `stopWatching`, `monitorDiskSpace`
- **Metadata**: `getFileInfo`, `setPermissions`, `getOwner`, `setOwner`
- **Traversal**: `walkDirectory`, `findFiles`, `findDirectories`
- **Content**: `readLines`, `writeLines`, `appendLines`
- **Comparison**: `compareFiles`, `syncDirectories`
- **Backup/Archive**: `backup`, `restore`, `compress`, `decompress`
- **Search**: `searchInFiles`, `indexDirectory`, `searchIndex`
- **Security**: `lockFile`, `unlockFile`, `getFileHash`, `validateFile`
- **Parallel**: `copyParallel`, `processFilesParallel`

**Total**: 40 functions

### 🌐 Network Module (`net.susu`)
**Import**: `net_module -> from <- import <- (server, websocket, smtp, ftp)`

Advanced networking and communication:
- **HTTP Server**: `createServer`, `stopServer`, `addRoute`, `addMiddleware`
- **WebSocket**: `createWebSocketServer`, `connectWebSocket`, `sendWebSocketMessage`
- **HTTP Client**: `httpWithAuth`, `httpWithRetry`, `httpBatch`, `downloadFile`, `uploadFile`
- **API Client**: `createApiClient`, `addHeader`, `setAuth`
- **Email**: `sendEmail`, `sendBulkEmail`
- **FTP**: `ftpConnect`, `ftpUpload`, `ftpDownload`, `ftpListFiles`
- **Network Utils**: `ping`, `traceroute`, `resolveHostname`, `getPublicIp`
- **Security**: `validateCertificate`, `getCertificateInfo`
- **Infrastructure**: `createProxy`, `createLoadBalancer`, `benchmarkLatency`

**Total**: 43 functions

### 📊 Data Science Module (`data.susu`)
**Import**: `data_module -> from <- import <- (dataframe, ml, stats, viz)`

Machine learning and analytics:
- **DataFrame**: `createDataFrame`, `readCsv`, `writeCsv`, `selectColumns`, `filterRows`
- **Statistics**: `describe`, `correlation`, `covariance`, `tTest`, `chiSquareTest`
- **ML Supervised**: `linearRegression`, `logisticRegression`, `randomForest`, `svm`, `neuralNetwork`
- **ML Unsupervised**: `kmeans`, `hierarchicalClustering`, `pca`, `dbscan`
- **Model Evaluation**: `trainTestSplit`, `crossValidate`, `evaluate`
- **Feature Engineering**: `scaleFeatures`, `encodeCategories`, `selectFeatures`
- **Time Series**: `timeSeries`, `decompose`, `forecast`, `detectAnomalies`
- **Visualization**: `plot`, `histogram`, `scatterPlot`, `boxPlot`, `heatmap`
- **Big Data**: `createSparkSession`, `readParquet`, `distributedCompute`

**Total**: 45 functions

### 🔐 Cryptography Module (`crypto.susu`)
**Import**: `crypto_module -> from <- import <- (hash, encrypt, sign, random)`

Security and encryption:
- **Hashing**: `sha256`, `sha512`, `md5`, `blake2b`
- **Symmetric Encryption**: `aesEncrypt`, `aesDecrypt`, `chachaEncrypt`, `chachaDecrypt`
- **Asymmetric Encryption**: `generateRsaKeyPair`, `rsaEncrypt`, `rsaDecrypt`
- **Digital Signatures**: `rsaSign`, `rsaVerify`, `ecdsaSign`, `ecdsaVerify`
- **Key Derivation**: `pbkdf2`, `scrypt`, `argon2`
- **Authentication**: `hmacSha256`, `hmacSha512`
- **Random Generation**: `randomBytes`, `randomInt`, `randomString`
- **Password Security**: `hashPassword`, `verifyPassword`, `generateSalt`
- **Certificates**: `parseCertificate`, `verifyCertificateChain`
- **JWT**: `createJwt`, `verifyJwt`, `decodeJwt`
- **Blockchain**: `generateMnemonic`, `mnemonicToSeed`, `deriveHdKey`

**Total**: 40 functions

## Usage Examples

### Basic Module Import
```susumu
// Import specific functions
math_module -> from <- import <- (factorial, isPrime, fibonacci)

// Use imported functions
5 -> factorial -> print         // 120
17 -> isPrime -> print          // true
10 -> fibonacci -> print        // 55
```

### Complex Module Integration
```susumu
// Multi-module data processing pipeline
crypto_module -> from <- import <- (sha256, randomBytes)
array_module -> from <- import <- (chunk, flatten, unique)
string_module -> from <- import <- (split, join, template)

data -> 
    split <- "," ->              // String module
    unique ->                    // Array module  
    chunk <- 3 ->               // Array module
    flatten ->                  // Array module
    join <- "|" ->              // String module
    sha256 ->                   // Crypto module
    print
```

### Parallel Processing Integration
```susumu
// Combine auto-available parallel functions with specialized modules
files -> readFilesParallel ->                    // Auto-available
    mapParallel <- "processContent" ->           // Auto-available  
    filterParallel <- "isValid" ->               // Auto-available
    reduce <- "combine" ->                       // Auto-available
    backup <- "/backup/processed" ->             // FS module
    print
```

## Performance Characteristics

### Auto-Available Functions
- **Instant Access**: No import overhead
- **Optimized**: Core functions highly optimized for common use cases
- **Parallel**: Key functions have parallel variants (`mapParallel`, `httpGetParallel`)
- **Memory Efficient**: Automatic resource management

### Standard Library Modules  
- **On-Demand Loading**: Only imported modules consume memory
- **Specialized**: Advanced algorithms optimized for specific domains
- **Enterprise Features**: Production-ready with comprehensive error handling
- **Extensible**: Easy to add domain-specific modules

## Best Practices

### 1. Use Auto-Available Functions First
```susumu
// Preferred: Use auto-available functions
data -> filter <- "isValid" -> map <- "transform" -> print

// Only import modules for advanced features
array_module -> from <- import <- (quickSort, binarySearch)
```

### 2. Import Only What You Need
```susumu
// Good: Specific imports
math_module -> from <- import <- (factorial, gcd)

// Avoid: Importing entire modules unnecessarily
// math_module -> from <- import <- *
```

### 3. Leverage Parallel Functions
```susumu
// Use parallel variants for large datasets
largeDataset -> mapParallel <- "expensiveOperation" -> print

// Use parallel I/O for multiple files  
fileList -> readFilesParallel -> print
```

### 4. Combine Modules Effectively
```susumu
// Chain operations across modules
crypto_module -> from <- import <- (sha256)
fs_module -> from <- import <- (backup)

files -> 
    readFilesParallel ->         // Auto-available
    mapParallel <- "validate" -> // Auto-available
    sha256 ->                    // Crypto module
    backup <- "/secure" ->       // FS module
    print
```

## Module Development Guidelines

### Creating New Modules
1. **Follow Naming Convention**: `module_name.susu`
2. **Use Arrow-Flow Syntax**: All functions use arrow chains
3. **Export Functions**: End with `(func1, func2, ...) -> export`
4. **Document Imports**: Include import examples in comments
5. **Provide Examples**: Show common usage patterns

### Example Module Template
```susumu
// Module Description - What this module provides
// Import: module_name -> from <- import <- (func1, func2)

func1(arg1, arg2) {
    (arg1, arg2) -> performOperation -> return
}

func2(data) {
    data -> processData -> return
}

// Export all functions
(func1, func2) -> export
```

## Future Modules

Planned standard library expansions:
- **Graphics Module**: 2D/3D graphics, image processing
- **Audio Module**: Sound processing, music generation
- **Game Module**: Game development utilities
- **Blockchain Module**: Expanded cryptocurrency support
- **AI Module**: Advanced AI/ML algorithms
- **Database Module**: SQL and NoSQL database connectors

---

**The Susumu Standard Library: Production-ready, comprehensive, and designed for the arrow-flow paradigm.**