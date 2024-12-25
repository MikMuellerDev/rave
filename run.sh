pw-loopback &
LOOPBACK_PID=$!

cargo r &
CARGO_PID=$!

while true; do
    if [ -d "/proc/${CARGO_PID}" ]; then
        echo "Waiting for rust to die..."
        sleep 1
    else
        break
    fi
done

kill "${LOOPBACK_PID}"
echo "Killed loopback."
