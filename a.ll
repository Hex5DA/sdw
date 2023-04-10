define i64 @main() {
  ; allocating 'a'
  %.1 = alloca i64
  store i64 10, ptr %.1
  ; conditional (if) statement begin
  ; dereferencing 'a'
  %.3 = load i64, ptr %.1
  %.4 = icmp slt i64 %.3, 5
  br i1 %.4, label %.5, label %.6
; true case
.5:
  ret i64 0
  br label %.2
; false case
.6:
  ; dereferencing 'a'
  %.7 = load i64, ptr %.1
  %.8 = icmp slt i64 %.7, 10
  br i1 %.8, label %.10, label %.9
.10:
  ret i64 6
  br label %.2
; false case
.9: 
  br label %.2
; conditional exit label
.2:
  ret i64 8
  unreachable
}

