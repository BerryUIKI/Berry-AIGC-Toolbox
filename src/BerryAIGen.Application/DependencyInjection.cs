using Microsoft.Extensions.DependencyInjection;

namespace BerryAIGen.Application;

/// <summary>
/// Dependency injection configuration for Application layer.
/// </summary>
public static class DependencyInjection
{
    /// <summary>
    /// Adds Application layer services to the DI container.
    /// </summary>
    public static IServiceCollection AddApplication(this IServiceCollection services)
    {
        // Register MediatR
        services.AddMediatR(cfg => 
        {
            cfg.RegisterServicesFromAssembly(typeof(DependencyInjection).Assembly);
        });

        // Register validators (when implemented)
        // services.AddValidatorsFromAssembly(typeof(DependencyInjection).Assembly);

        // Register mapping profiles (when implemented)
        // services.AddAutoMapper(typeof(DependencyInjection).Assembly);

        return services;
    }
}