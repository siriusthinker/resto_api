import http.client
import json
import concurrent.futures
import random

def send_request(method, path, body=None):
    connection = http.client.HTTPConnection("127.0.0.1", 8080)
    headers = {"Content-type": "application/json"}

    if body:
        body = json.dumps(body)

    connection.request(method, path, body, headers)
    response = connection.getresponse()
    return response

def add_random_order():
    item = random.randint(1, 21) # random.choice([1, 2, 3, 4, 5, 6, 7, 8, 9])
    table_number = random.randint(1, 20)

    order_data = {
        "items": [item],
        "table_id": table_number
    }

    response = send_request("POST", "/orders", body=order_data)
    # print(f'{order_data} {response}')
    print(f"Add Order Response: {response.read().decode()}")

def query_random_orders():
    table_number = random.randint(1, 99)
    response = send_request("GET", f"/orders/{table_number}")
    print(f"Query Orders for Table {table_number} Response: {response.read().decode()}")

def remove_random_item():
    table_number = random.randint(1, 99)
    item = random.randint(1, 21)

    response = send_request("DELETE", f"/orders/{table_number}/{item}")
    print(f"Remove Item Response: {response.read().decode()}")

# Use ThreadPoolExecutor to make concurrent requests
with concurrent.futures.ThreadPoolExecutor(max_workers=20) as executor:
    # Add 10-20 concurrent random add order requests
    futures = [executor.submit(add_random_order) for _ in range(random.randint(10, 20))]
    concurrent.futures.wait(futures)

    # Add 10-20 concurrent random query orders requests
    futures = [executor.submit(query_random_orders) for _ in range(random.randint(10, 20))]
    concurrent.futures.wait(futures)

    # Add 10-20 concurrent random remove item requests
    futures = [executor.submit(remove_random_item) for _ in range(random.randint(10, 20))]
    concurrent.futures.wait(futures)


