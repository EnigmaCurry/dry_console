(
    set -e
    echo "Hii" >/dev/stderr
    for i in $(seq 100); do
        echo $i
        sleep 0.1
    done
)
