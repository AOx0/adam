# # Verify rules start empty
# GET http://localhost/firewall/rules
# HTTP 200
# [Asserts]
# header "Content-Type" == "application/json"
# body == "[]"

# Submit a rule
POST http://localhost/firewall/add
Content-Type: application/json
{
    "name": "Disable pings",
    "description": "Forbidden ICMPs",
    "rule": {
        "action":"drop",
        "matches":{"protocol":"ICMP"},
        "applies_to":"source"
    }
}
HTTP 200
[Captures]
rule-id: jsonpath "$['id']"

POST http://localhost/firewall/add
Content-Type: application/json
{
    "name": "Block 8080",
    "description": "Block access to port 8008",
    "rule": {
        "action":"drop",
        "matches":{ "port": 8080 },
        "applies_to":"source"
    }
}
HTTP 200
[Captures]
rule-id2: jsonpath "$['id']"

GET http://localhost/firewall/rule/{{rule-id}}
HTTP 200
GET http://localhost/firewall/rule/{{rule-id2}}
HTTP 200

POST http://localhost/firewall/enable/{{rule-id}}
HTTP 200
POST http://localhost/firewall/enable/{{rule-id2}}
HTTP 200

POST http://localhost/firewall/start
HTTP 200
