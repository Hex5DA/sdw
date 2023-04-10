define i64 @second() {
  ret i64 5
  unreachable
}
define i64 @main() {
  ; allocating 'a'
  %.1 = alloca i64
  %.2 = call i64 @second()
  store i64 %.2, ptr %.1
  ; dereferencing 'a'
  %.3 = load i64, ptr %.1
  ret i64 %.3
  unreachable
}

