;;; example-custom-app.scm — Parses bracketed application logs
;;; Format: "[ERROR] 2024-01-15 10:30:00 | MyService | Connection timeout to db:5432"

(define (parser-name) "custom-app")
(define (parser-description) "Bracketed application log parser with pipe delimiters")

(define (detect raw)
  (regex-match? "^\\[(INFO|WARN|ERROR|DEBUG)\\]" raw))

(define (parse raw)
  (let* ((captures (regex-captures
                     "^\\[(\\w+)\\]\\s+(\\S+\\s+\\S+)\\s+\\|\\s+(\\S+)\\s+\\|\\s+(.+)"
                     raw))
         (severity  (if (> (length captures) 0) (car captures) "INFO"))
         (timestamp (if (> (length captures) 1) (cadr captures) (now-utc)))
         (service   (if (> (length captures) 2) (caddr captures) "unknown"))
         (message   (if (> (length captures) 3) (cadddr captures) raw)))
    (hash "timestamp" timestamp
          "severity"  (string-upper severity)
          "service"   service
          "message"   message)))
