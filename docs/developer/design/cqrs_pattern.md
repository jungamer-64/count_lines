# CQRS Pattern Notes

The application layer separates write and read concerns:

- `application/commands` contains orchestrators such as `RunAnalysisCommand`.
- `application/queries` exposes query services for configuration discovery.

Commands depend on ports that describe the infrastructure services required to
perform a use case. Queries remain side-effect free and cacheable.
