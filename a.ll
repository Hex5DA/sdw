define i64 @main() {
  ; allocating 'v'
  %v.0 = alloca i64
  store i64 5, ptr %v.0
  ; allocating 'b'
  %b.0 = alloca i64
  ; dereferencing 'v'
  %v.1 = load i64, ptr %v.0
  store i64 %v.1, ptr %b.0
  ; allocating 'c'
  %c.0 = alloca i64
  ; dereferencing 'b'
  %b.1 = load i64, ptr %b.0
  store i64 %b.1, ptr %c.0
  ; dereferencing 'c'
  %c.1 = load i64, ptr %c.0
  ret i64 %c.1
}

