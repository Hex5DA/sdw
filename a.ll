define i64 @main() {
  br i1 1, label %.2, label %.3
.2:
  ret i64 0
  br label %.1
.3: 
  ret i64 2
  br label %.1
.1:
  ret i64 4
  unreachable
}

