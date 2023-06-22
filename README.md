# Schluessel 🔑
Schluessel serves as a straightforward entry point for accessing protected services. It presents users with a password entry field. Once the user enters the password, they are presented with a list of services that have been registered by Schloss services with the specific Schluessel instance. Users can choose any of these services and automatically receive an authentication token for the corresponding domain.

## Important
It's important to note that Schloss & Schluessel primarily serve as a means to hide resources from crawlers and prevent accidental access. They do not offer robust security akin to a simple password protection. If you have higher security requirements, it is strongly recommended to reinforce the security of your services and consider implementing a dedicated authentication proxy such as OAuth2-proxy.

## About Schloss & Schluessel 🔐
Schloss and Schluessel are two complementary services. Schluessel serves as the user's entry point, allowing them to log in and access all the services registered by Schloss instances across different clusters.

Schloss & Schluessel aim to utilize the features of Istio while minimizing the added complexity of OAuth and user management, without requiring modifications to the services. It's important to note that Schloss & Schluessel do not provide robust security for your services. They simply provide a level of authentication that may be considered weak.

The user flow is as follows:

```mermaid
sequenceDiagram
    box Control Cluster
    participant Schluessel
    end
    participant User
    box Application Cluster
    participant Schloss
    participant Istio
    end
    User->>Schluessel: Enter Password
    Schluessel-->>User: Present All Services
    User->>Schluessel: Clicks on desired service
    Schluessel->>Schloss: Open Schloss with URL & Auth Token
    Schloss-->>User: Set Session Cookie & Send Redirect to desired service
    User->>Istio: Open Service with  Session Cookie
```

The service autodiscovery operates as follows:

```mermaid
sequenceDiagram
    box Application Cluster
    participant Istio
    participant Schluessel
    end
    box Control Cluster
    participant Schloss
    end
    Schloss->>Istio: Discover available VirtualServices
    Schloss->>Schluessel: Register Services
```

### Environment Variables

Environment variables can be configured in the docker-compose.yml file or Helms values.yaml.

| Variable | Default | Description |
| --- | --- | --- |
| PASSWORD | ```password``` | (Secret) The password to use for authentication. |
| SHARED_SECRET | ```shared_secret``` | (Secret) The shared secret Schluessel authenticates against Schloss. |
| HTTP_HOST | ```127.0.0.1``` | The host of the default bind address. |
| HTTP_PORT | ```8080``` | The port of the default bind address. |

### API

| Method | Endpoint | Description |
| --- | --- | --- |
| GET | / | Returns an HTML form for password input. |
| POST | /authenticate | Accepts a password, if valid, returns an HTML page with registered services, else returns an Unauthorized status. |
| POST | /register | Registers a Schloss instance with its domain and services. Accepts and returns a JSON structure with domain and services.

## How to run it

### Locally for development purpose
The reommended method of running Schloss locally is by utilizing docker-compose.

Schloss can be run locally by utilizing the provided ```local.yaml``` via ```docker compose -f local.yaml up```. Afterwards there are available:
- A Schluessel under http://127.0.0.1:8080
- A Schloss under http://127.0.0.1:8081
- A Schloss under http://127.0.0.1:8082

After authenticating on Schluessel the user can open one of the services and check in the browser if the auth cookie was successfully set.

You can also run Schloss und Schluessel locally via ```cargo run`` but this is much more complex.

### In a k8s Cluster
For running schloss productively you need knowledge of Helm, Kubernetes and Istio. Ideally Schluessel is hosted on a different Cluster than Schloss. Both services can be installed via their respective Helm charts under ```deploy/helm```. Before doing so please modify the values.yaml to fit your needs.

If you utilize Istio for authentication as recommended then please take into account that you need to create VirtualServices to protect your resources via cookie matching.

An example for such a VirtualService could look like this:
```yaml
apiVersion: networking.istio.io/v1alpha3
kind: VirtualService
metadata:
  name: cookie-vs-example
spec:
  hosts:
  - "*"
  gateways:
  - my-gateway
  http:
  - match:
    - uri:
        prefix: /
      headers:
        cookie:
          regex: ^(.\*?;)?(token_name=token_value)(;.\*)?$
    route:
    - destination:
        host: protected-service
```