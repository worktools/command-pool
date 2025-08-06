#!/bin/bash
SLEEP_TIME=$(( ( RANDOM % 7 ) + 3 ))
echo "Sleeping for ${SLEEP_TIME} seconds..."
sleep ${SLEEP_TIME}
echo "Task finished after ${SLEEP_TIME} seconds."
