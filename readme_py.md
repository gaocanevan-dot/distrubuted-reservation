python client.py -s 127.0.0.1:5000 query -n facility1 -d monday tuesday
python client.py -s 127.0.0.1:5000 book --name facility1 -d monday -s 2 --num-slots 2 -u 1
python client.py -s 127.0.0.1:5000 update -c 1 -o 2
python client.py -s 127.0.0.1:5000 monitor -d 60