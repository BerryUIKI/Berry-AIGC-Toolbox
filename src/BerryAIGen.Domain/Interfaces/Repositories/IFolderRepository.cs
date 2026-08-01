using BerryAIGen.Domain.Entities;
using BerryAIGen.Domain.ValueObjects;

namespace BerryAIGen.Domain.Interfaces.Repositories;

/// <summary>
/// Repository interface for Folder aggregate root.
/// </summary>
public interface IFolderRepository
{
    /// <summary>
    /// Gets a folder by its ID.
    /// </summary>
    Task<Folder?> GetByIdAsync(FolderId id, CancellationToken cancellationToken = default);

    /// <summary>
    /// Gets a folder by its path.
    /// </summary>
    Task<Folder?> GetByPathAsync(FilePath path, CancellationToken cancellationToken = default);

    /// <summary>
    /// Gets all root folders (folders without parent).
    /// </summary>
    Task<List<Folder>> GetRootFoldersAsync(CancellationToken cancellationToken = default);

    /// <summary>
    /// Gets all subfolders of a specific folder.
    /// </summary>
    Task<List<Folder>> GetSubfoldersAsync(FolderId parentFolderId, CancellationToken cancellationToken = default);

    /// <summary>
    /// Gets all folders.
    /// </summary>
    Task<List<Folder>> GetAllAsync(CancellationToken cancellationToken = default);

    /// <summary>
    /// Gets all watched folders.
    /// </summary>
    Task<List<Folder>> GetWatchedFoldersAsync(CancellationToken cancellationToken = default);

    /// <summary>
    /// Adds a new folder.
    /// </summary>
    Task AddAsync(Folder folder, CancellationToken cancellationToken = default);

    /// <summary>
    /// Updates an existing folder.
    /// </summary>
    void Update(Folder folder);

    /// <summary>
    /// Deletes a folder.
    /// </summary>
    void Delete(Folder folder);

    /// <summary>
    /// Gets the count of all folders.
    /// </summary>
    Task<int> GetCountAsync(CancellationToken cancellationToken = default);

    /// <summary>
    /// Checks if a folder exists with the given path.
    /// </summary>
    Task<bool> ExistsByPathAsync(FilePath path, CancellationToken cancellationToken = default);
}