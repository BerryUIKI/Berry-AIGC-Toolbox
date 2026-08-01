using BerryAIGen.Domain.Entities;

namespace BerryAIGen.Domain.Interfaces.Repositories;

/// <summary>
/// Repository interface for Album aggregate root.
/// </summary>
public interface IAlbumRepository
{
    /// <summary>
    /// Gets an album by its ID.
    /// </summary>
    Task<Album?> GetByIdAsync(AlbumId id, CancellationToken cancellationToken = default);

    /// <summary>
    /// Gets an album by its name.
    /// </summary>
    Task<Album?> GetByNameAsync(string name, CancellationToken cancellationToken = default);

    /// <summary>
    /// Gets all albums.
    /// </summary>
    Task<List<Album>> GetAllAsync(CancellationToken cancellationToken = default);

    /// <summary>
    /// Gets all albums containing a specific image.
    /// </summary>
    Task<List<Album>> GetByImageIdAsync(ImageId imageId, CancellationToken cancellationToken = default);

    /// <summary>
    /// Adds a new album.
    /// </summary>
    Task AddAsync(Album album, CancellationToken cancellationToken = default);

    /// <summary>
    /// Updates an existing album.
    /// </summary>
    void Update(Album album);

    /// <summary>
    /// Deletes an album.
    /// </summary>
    void Delete(Album album);

    /// <summary>
    /// Gets the count of all albums.
    /// </summary>
    Task<int> GetCountAsync(CancellationToken cancellationToken = default);

    /// <summary>
    /// Checks if an album exists with the given name.
    /// </summary>
    Task<bool> ExistsByNameAsync(string name, CancellationToken cancellationToken = default);
}