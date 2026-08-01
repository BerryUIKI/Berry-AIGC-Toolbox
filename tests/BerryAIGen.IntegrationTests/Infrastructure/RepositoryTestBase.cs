using BerryAIGen.Infrastructure.Data.Context;
using Microsoft.EntityFrameworkCore;
using System.IO;

namespace BerryAIGen.IntegrationTests.Infrastructure;

/// <summary>
/// Base class for repository integration tests.
/// Provides a fresh SQLite in-memory database for each test.
/// </summary>
public abstract class RepositoryTestBase : IDisposable
{
    protected readonly SqliteDbContext DbContext;
    private readonly string _databasePath;

    protected RepositoryTestBase()
    {
        // Create a unique temporary database file for each test
        _databasePath = Path.Combine(Path.GetTempPath(), $"test_{Guid.NewGuid()}.db");

        var options = new DbContextOptionsBuilder<SqliteDbContext>()
            .UseSqlite($"Data Source={_databasePath}")
            .Options;

        DbContext = new SqliteDbContext(options);

        // Ensure database is created
        DbContext.Database.EnsureCreated();
    }

    public void Dispose()
    {
        DbContext?.Dispose();

        // Clean up the temporary database file
        if (File.Exists(_databasePath))
        {
            try
            {
                File.Delete(_databasePath);
            }
            catch
            {
                // Ignore cleanup errors
            }
        }

        GC.SuppressFinalize(this);
    }
}