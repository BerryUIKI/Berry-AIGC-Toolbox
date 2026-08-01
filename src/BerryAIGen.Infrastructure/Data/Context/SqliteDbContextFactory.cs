using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Design;

namespace BerryAIGen.Infrastructure.Data.Context;

/// <summary>
/// Design-time factory for SqliteDbContext for EF Core migrations.
/// </summary>
public class SqliteDbContextFactory : IDesignTimeDbContextFactory<SqliteDbContext>
{
    /// <summary>
    /// Creates a new instance of SqliteDbContext for design-time operations.
    /// </summary>
    public SqliteDbContext CreateDbContext(string[] args)
    {
        var optionsBuilder = new DbContextOptionsBuilder<SqliteDbContext>();
        
        // Use a default connection string for migrations
        // In production, this will be overridden by the actual connection string
        optionsBuilder.UseSqlite("Data Source=berryaigen.db");
        
        return new SqliteDbContext(optionsBuilder.Options);
    }
}