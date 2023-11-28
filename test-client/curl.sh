for i in {1..50}
do
  curl --location 'localhost:3000' \
  --header 'Content-Type: application/json' \
  --data '{"prompt":"test'$i'"}'
done
