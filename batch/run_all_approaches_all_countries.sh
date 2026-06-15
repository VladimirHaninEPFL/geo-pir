
cd /home/hanin/geo-pir/batch

# clean output directory
rm /home/hanin/geo-pir/output/*

# start with spiral
COUNTRY=Switzerland
ARCHI=Spiral
START=649891036
DEST=(312462415 296962379)

for APPROACH in node0 node1 node2 node3 block0.1 block0.25 block0.5 block1; do 
    sbatch run_small.sh $COUNTRY $ARCHI $APPROACH $START "${DEST[@]}"
    sleep 1
done

COUNTRY=France
ARCHI=Spiral
START=249481666
DEST=(12625261402 2106958155)

for APPROACH in node0 node1 node2 node3 block0.1 block0.25 block0.5 block1; do 
    sbatch run_large.sh $COUNTRY $ARCHI $APPROACH $START "${DEST[@]}"
    sleep 1
done

# now do Sindlepass

# compile the go code of singlepass !
cd /home/hanin/SinglePass/
go build -o singlepass-client ./cmd/singlepass_demo_node/client/client.go
go build -o singlepass-server ./cmd/singlepass_demo_node/server/server.go

cd /home/hanin/geo-pir/batch

COUNTRY=Switzerland
ARCHI=SinglePass
START=649891036
DEST=(312462415 296962379)

for APPROACH in node0 node1 node2 node3 block0.1 block0.25 block0.5 block1; do 
    sbatch run_small.sh $COUNTRY $ARCHI $APPROACH $START "${DEST[@]}"
    sleep 1
done

COUNTRY=France
ARCHI=SinglePass
START=249481666
DEST=(12625261402 2106958155)

for APPROACH in node0 node1 node2 node3 block0.1 block0.25 block0.5 block1; do 
    sbatch run_large.sh $COUNTRY $ARCHI $APPROACH $START "${DEST[@]}"
    sleep 1
done