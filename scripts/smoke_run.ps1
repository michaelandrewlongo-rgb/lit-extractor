param(
  [string]$Query = "neurosurgery operative anatomy",
  [string]$Since = "30d",
  [int]$Limit = 100
)

cargo run -- run --query $Query --since $Since --limit $Limit
if ($LASTEXITCODE -ne 0) {
  throw "lit run failed"
}

cargo run -- qa
