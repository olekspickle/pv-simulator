# pv-simulator

The idea is to simulate PV values interaction of Solar panels and consumption.

Consuption is measured by **meter** and then added to RabbitMQ instance.
From the rmq instance *PV generator* then takes values of a meter
and outputs summarized values.

The simulation should record each second with the meter values 0-9000 Watts.

The end result of a simulation is a CSV file.

