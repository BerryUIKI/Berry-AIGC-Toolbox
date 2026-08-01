using BerryAIGen.Domain.Entities;
using BerryAIGen.Domain.ValueObjects;

namespace BerryAIGen.Domain.Interfaces.Repositories;

/// <summary>
/// Repository interface for Image aggregate root.
/// </summary>
public interface IImageRepository
{
    /// <summary>
    /// Gets an image by its ID.
    /// </summary>
    Task<Image?> GetByIdAsync(ImageId id, CancellationToken cancellationToken = default);

    /// <summary>
    /// Gets an image by its file hash.
    /// </summary>
    Task<Image?> GetByHashAsync(Hash hash, CancellationToken cancellationToken = default);

    /// <summary>
    /// Gets an image by its file path.
    /// </summary>
    Task<Image?> GetByPathAsync(FilePath path, CancellationToken cancellationToken = default);

    /// <summary>
    /// Gets all images in a specific folder.
    /// </summary>
    Task<List<Image>> GetByFolderIdAsync(FolderId folderId, CancellationToken cancellationToken = default);

    /// <summary>
    /// Gets all images with specific tag.
    /// </summary>
    Task<List<Image>> GetByTagIdAsync(TagId tagId, CancellationToken cancellationToken = default);

    /// <summary>
    /// Gets all images in an album.
    /// </summary>
    Task<List<Image>> GetByAlbumIdAsync(AlbumId albumId, CancellationToken cancellationToken = default);

    /// <summary>
    /// Gets all images marked as favorites.
    /// </summary>
    Task<List<Image>> GetFavoritesAsync(CancellationToken cancellationToken = default);

    /// <summary>
    /// Gets all images marked as NSFW.
    /// </summary>
    Task<List<Image>> GetNSFWAsync(CancellationToken cancellationToken = default);

    /// <summary>
    /// Searches images by metadata criteria.
    /// </summary>
    Task<List<Image>> SearchAsync(
        string? query,
        List<TagId>? tagIds,
        Rating? minRating,
        bool? favoritesOnly,
        bool? nsfwOnly,
        CancellationToken cancellationToken = default);

    /// <summary>
    /// Adds a new image.
    /// </summary>
    Task AddAsync(Image image, CancellationToken cancellationToken = default);

    /// <summary>
    /// Updates an existing image.
    /// </summary>
    void Update(Image image);

    /// <summary>
    /// Deletes an image.
    /// </summary>
    void Delete(Image image);

    /// <summary>
    /// Gets the count of all images.
    /// </summary>
    Task<int> GetCountAsync(CancellationToken cancellationToken = default);

    /// <summary>
    /// Gets the count of images in a folder.
    /// </summary>
    Task<int> GetCountByFolderIdAsync(FolderId folderId, CancellationToken cancellationToken = default);

    /// <summary>
    /// Checks if an image exists with the given hash.
    /// </summary>
    Task<bool> ExistsByHashAsync(Hash hash, CancellationToken cancellationToken = default);

    /// <summary>
    /// Checks if an image exists with the given path.
    /// </summary>
    Task<bool> ExistsByPathAsync(FilePath path, CancellationToken cancellationToken = default);
}