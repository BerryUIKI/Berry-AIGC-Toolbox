using BerryAIGen.Domain.Entities;
using BerryAIGen.Domain.Interfaces.Repositories;
using BerryAIGen.Domain.ValueObjects;
using Microsoft.EntityFrameworkCore;

namespace BerryAIGen.Infrastructure.Data.Repositories;

/// <summary>
/// Repository implementation for Album aggregate root using Entity Framework Core.
/// </summary>
public class AlbumRepository : IAlbumRepository
{
    private readonly Context.SqliteDbContext _context;

    /// <summary>
    /// Initializes a new instance of the AlbumRepository class.
    /// </summary>
    public AlbumRepository(Context.SqliteDbContext context)
    {
        _context = context ?? throw new ArgumentNullException(nameof(context));
    }

    /// <inheritdoc/>
    public async Task<Album?> GetByIdAsync(AlbumId id, CancellationToken cancellationToken = default)
    {
        return await _context.Albums
            .FirstOrDefaultAsync(e => e.Id == id, cancellationToken);
    }

    /// <inheritdoc/>
    public async Task<Album?> GetByNameAsync(string name, CancellationToken cancellationToken = default)
    {
        return await _context.Albums
            .FirstOrDefaultAsync(e => e.Name == name, cancellationToken);
    }

    /// <inheritdoc/>
    public async Task<List<Album>> GetAllAsync(CancellationToken cancellationToken = default)
    {
        return await _context.Albums
            .OrderBy(e => e.Name)
            .ToListAsync(cancellationToken);
    }

    /// <inheritdoc/>
    public async Task<List<Album>> GetByImageIdAsync(ImageId imageId, CancellationToken cancellationToken = default)
    {
        // Query through the AlbumImages join table
        var albumIds = await _context.Set<Dictionary<string, object>>("AlbumImages")
            .Where(ai => (Guid)ai["ImageId"] == imageId)
            .Select(ai => (Guid)ai["AlbumId"])
            .ToListAsync(cancellationToken);

        return await _context.Albums
            .Where(e => albumIds.Contains(e.Id))
            .OrderBy(e => e.Name)
            .ToListAsync(cancellationToken);
    }

    /// <inheritdoc/>
    public async Task AddAsync(Album album, CancellationToken cancellationToken = default)
    {
        await _context.Albums.AddAsync(album, cancellationToken);
    }

    /// <inheritdoc/>
    public void Update(Album album)
    {
        _context.Albums.Update(album);
    }

    /// <inheritdoc/>
    public void Delete(Album album)
    {
        _context.Albums.Remove(album);
    }

    /// <inheritdoc/>
    public async Task<int> GetCountAsync(CancellationToken cancellationToken = default)
    {
        return await _context.Albums.CountAsync(cancellationToken);
    }

    /// <inheritdoc/>
    public async Task<bool> ExistsByNameAsync(string name, CancellationToken cancellationToken = default)
    {
        return await _context.Albums
            .AnyAsync(e => e.Name == name, cancellationToken);
    }
}