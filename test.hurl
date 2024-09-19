# Verify rules start empty
GET http://localhost/firewall/rules
HTTP 200
[Asserts]
header "Content-Type" == "application/json"
body == "[]"

# Submit a rule
POST http://localhost/firewall/add
Content-Type: application/json
{
    "action":"drop",
    "matches":{"protocol":"ICMP"},
    "applies_to":"source"
}
HTTP 200
[Captures]
rule-id: jsonpath "$['id']"

GET http://localhost/firewall/rule/{{rule-id}}
HTTP 200
[Asserts]
header "Content-Type" == "application/json"
{"action":"drop","matches":{"protocol":"ICMP"},"applies_to":"source","enabled":true,"init":true}

POST http://localhost/firewall/enable/{{rule-id}}
HTTP 200
