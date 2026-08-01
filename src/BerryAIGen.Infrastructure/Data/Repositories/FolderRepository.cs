using BerryAIGen.Domain.Entities;
using BerryAIGen.Domain.Interfaces.Repositories;
using BerryAIGen.Domain.ValueObjects;
using Microsoft.EntityFrameworkCore;

namespace BerryAIGen.Infrastructure.Data.Repositories;

/// <summary>
/// Repository implementation for Folder aggregate root using Entity Framework Core.
/// </summary>
public class FolderRepository : IFolderRepository
{
    private readonly Context.SqliteDbContext _context;

    /// <summary>
    /// Initializes a new instance of the FolderRepository class.
    /// </summary>
    public FolderRepository(Context.SqliteDbContext context)
    {
        _context = context ?? throw new ArgumentNullException(nameof(context));
    }

    /// <inheritdoc/>
    public async Task<Folder?> GetByIdAsync(FolderId id, CancellationToken cancellationToken = default)
    {
        return await _context.Folders
            .FirstOrDefaultAsync(e => e.Id == id, cancellationToken);
    }

    /// <inheritdoc/>
    public async Task<Folder?> GetByPathAsync(FilePath path, CancellationToken cancellationToken = default)
    {
        return await _context.Folders
            .FirstOrDefaultAsync(e => e.Path == path, cancellationToken);
    }

    /// <inheritdoc/>
    public async Task<List<Folder>> GetRootFoldersAsync(CancellationToken cancellationToken = default)
    {
        return await _context.Folders
            .Where(e => e.ParentFolderId == null)
            .OrderBy(e => e.Path)
            .ToListAsync(cancellationToken);
    }

    /// <inheritdoc/>
    public async Task<List<Folder>> GetSubfoldersAsync(FolderId parentFolderId, CancellationToken cancellationToken = default)
    {
        return await _context.Folders
            .Where(e => e.ParentFolderId == parentFolderId)
            .OrderBy(e => e.Path)
            .ToListAsync(cancellationToken);
    }

    /// <inheritdoc/>
    public async Task<List<Folder>> GetAllAsync(CancellationToken cancellationToken = default)
    {
        return await _context.Folders
            .OrderBy(e => e.Path)
            .ToListAsync(cancellationToken);
    }

    /// <inheritdoc/>
    public async Task<List<Folder>> GetWatchedFoldersAsync(CancellationToken cancellationToken = default)
    {
        return await _context.Folders
            .Where(e => e.IsWatched)
            .OrderBy(e => e.Path)
            .ToListAsync(cancellationToken);
    }

    /// <inheritdoc/>
    public async Task AddAsync(Folder folder, CancellationToken cancellationToken = default)
    {
        await _context.Folders.AddAsync(folder, cancellationToken);
    }

    /// <inheritdoc/>
    public void Update(Folder folder)
    {
        _context.Folders.Update(folder);
    }

    /// <inheritdoc/>
    public void Delete(Folder folder)
    {
        _context.Folders.Remove(folder);
    }

    /// <inheritdoc/>
    public async Task<int> GetCountAsync(CancellationToken cancellationToken = default)
    {
        return await _context.Folders.CountAsync(cancellationToken);
    }

    /// <inheritdoc/>
    public async Task<bool> ExistsByPathAsync(FilePath path, CancellationToken cancellationToken = default)
    {
        return await _context.Folders
            .AnyAsync(e => e.Path == path, cancellationToken);
    }
}