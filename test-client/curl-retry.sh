# for prompt in a list of string

task_ids=(
    "1702726229340-133384"
    "1702726229312-300556"
    "1702726229275-768300"
    "1702726229248-815422"
    "1702726229203-107139"
    "1702726228891-956695"
    "1702726228616-700895"
    "1702726228460-130511"
    "1702726228387-440097"
    "1702726228379-611839"
    "1702726228359-446487"
    "1702726228304-905798"
    "1702726228287-396368"
    "1702726228268-268392"
    "1702726228204-005176"
    "1702726228149-973024"
    "1702726228139-988398"
    "1702726228094-548463"
    "1702726228084-023380"
    "1702726228074-829415"
    "1702726228011-091126"
    "1702726227982-565943"
    "1702726227973-258617"
    "1702726227964-630131"
    "1702726227955-045905"
    "1702726227928-295916"
    "1702726227890-237749"
    "1702726227779-056312"
    "1702726227713-439754"
    "1702726227612-941837"
    "1702726227557-729537"
    "1702726227521-991689"
    "1702726227512-321153"
    "1702726227466-684458"
    "1702726227457-487091"
    "1702726227421-511638"
    "1702726227412-216513"
    "1702726227366-033974"
    "1702726227357-151717"
    "1702726227346-866999"
    "1702726227337-253980"
    "1702726227291-219310"
    "1702726227282-230373"
    "1702726227254-921237"
    "1702726227244-375685"
    "1702726227236-225112"
    "1702726227227-933627"
    "1702726227206-674857"
    "1702726227160-176238"
    "1702726227151-818306"
    "1702726227134-967169"
    "1702726227125-466233"
    "1702726227114-347793"
    "1702726227104-512663"
    "1702726227095-013389"
    "1702726227086-373541"
    "1702726227076-931459"
    "1702726227066-193169"
    "1702726227049-638879"
    "1702726227040-645454"
    "1702726227016-191528"
    "1702726226994-895355"
    "1702726226974-768431"
    "1702726226915-886985"
    "1702726226906-781641"
    "1702726226896-741939"
    "1702726226886-984480"
    "1702726226820-305523"
    "1702726226810-867701"
    "1702726226800-791366"
    "1702726226790-694838"
    "1702726226780-423897"
    "1702726226735-249653"
    "1702726226716-848863"
    "1702726226696-656802"
    "1702726226581-973303"
    "1702726226571-498619"
    "1702726226561-745535"
    "1702726226504-791042"
    "1702726226459-548712"
    "1702726226441-488146"
    "1702726226404-864214"
    "1702726226386-660420"
    "1702726226356-927037"
    "1702726226271-229707"
    "1702726226244-946937"
)

for task_id in "${task_ids[@]}"
do
    echo "$task_id"
    curl "http://localhost:3000/api/yum/generate/result/$task_id/retry" --request POST
    echo ""
    sleep 1
done
