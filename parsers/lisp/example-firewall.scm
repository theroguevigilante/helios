;;; example-firewall.scm — Helios Lisp Parser Extension
;;; Parses logs like: "2024-01-15T10:30:00Z FIREWALL DENY src=192.168.1.1 dst=10.0.0.1 proto=TCP"

(define (parser-name) "custom-firewall")
(define (parser-description) "Custom firewall ALLOW/DENY log parser")

(define (detect raw)
  (and (string-contains? raw "FIREWALL")
       (or (string-contains? raw "DENY")
           (string-contains? raw "ALLOW"))))

(define (parse raw)
  (let* ((parts    (helios/split raw " "))
         (len      (length parts))
         (ts       (if (> len 0) (car parts) (now-utc)))
         (action   (if (> len 2) (caddr parts) "UNKNOWN"))
         (severity (if (equal? action "DENY") "WARN" "INFO")))
    (hash "timestamp" ts
          "severity"  severity
          "service"   "firewall"
          "hostname"  "edge-gw"
          "message"   (string-trim raw))))
