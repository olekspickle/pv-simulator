# pv-simulator

## Overview

The idea is to simulate **PV generator** values interaction of Solar panels
and building consumption measured by a Meter.

Consuption is measured by **meter** and then added to RabbitMQ instance.
From the rmq instance **PV generator** then takes values of a meter
and outputs summarized values.

The simulation should record each second with the meter values 0-9000 Watts.

The end result of a simulation is a CSV file.

## How to run

Run `start.sh` or manually:

1. Run rabbitmq:
    ```bash
        docker compose up
    ```

2. Run the app:
    ```bash
        RUST_LOG=info cargo run
    ```


