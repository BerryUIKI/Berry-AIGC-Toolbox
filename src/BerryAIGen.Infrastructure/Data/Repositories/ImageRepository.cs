using BerryAIGen.Domain.Entities;
using BerryAIGen.Domain.Interfaces.Repositories;
using BerryAIGen.Domain.ValueObjects;
using Microsoft.EntityFrameworkCore;
using System.Linq.Expressions;

namespace BerryAIGen.Infrastructure.Data.Repositories;

/// <summary>
/// Repository implementation for Image aggregate root using Entity Framework Core.
/// </summary>
public class ImageRepository : IImageRepository
{
    private readonly Context.SqliteDbContext _context;

    /// <summary>
    /// Initializes a new instance of the ImageRepository class.
    /// </summary>
    public ImageRepository(Context.SqliteDbContext context)
    {
        _context = context ?? throw new ArgumentNullException(nameof(context));
    }

    /// <inheritdoc/>
    public async Task<Image?> GetByIdAsync(ImageId id, CancellationToken cancellationToken = default)
    {
        return await _context.Images
            .FirstOrDefaultAsync(e => e.Id == id, cancellationToken);
    }

    /// <inheritdoc/>
    public async Task<Image?> GetByHashAsync(Hash hash, CancellationToken cancellationToken = default)
    {
        return await _context.Images
            .FirstOrDefaultAsync(e => e.Hash == hash, cancellationToken);
    }

    /// <inheritdoc/>
    public async Task<Image?> GetByPathAsync(FilePath path, CancellationToken cancellationToken = default)
    {
        return await _context.Images
            .FirstOrDefaultAsync(e => e.Path == path, cancellationToken);
    }

    /// <inheritdoc/>
    public async Task<List<Image>> GetByFolderIdAsync(FolderId folderId, CancellationToken cancellationToken = default)
    {
        return await _context.Images
            .Where(e => e.FolderId == folderId)
            .OrderByDescending(e => e.CreatedAt)
            .ToListAsync(cancellationToken);
    }

    /// <inheritdoc/>
    public async Task<List<Image>> GetByTagIdAsync(TagId tagId, CancellationToken cancellationToken = default)
    {
        // Query through the ImageTags join table
        var imageIds = await _context.Set<Dictionary<string, object>>("ImageTags")
            .Where(it => (Guid)it["TagId"] == tagId)
            .Select(it => (Guid)it["ImageId"])
            .ToListAsync(cancellationToken);

        return await _context.Images
            .Where(e => imageIds.Contains(e.Id))
            .OrderByDescending(e => e.CreatedAt)
            .ToListAsync(cancellationToken);
    }

    /// <inheritdoc/>
    public async Task<List<Image>> GetByAlbumIdAsync(AlbumId albumId, CancellationToken cancellationToken = default)
    {
        // Query through the AlbumImages join table
        var imageIds = await _context.Set<Dictionary<string, object>>("AlbumImages")
            .Where(ai => (Guid)ai["AlbumId"] == albumId)
            .OrderBy(ai => (int)ai["SortOrder"])
            .Select(ai => (Guid)ai["ImageId"])
            .ToListAsync(cancellationToken);

        return await _context.Images
            .Where(e => imageIds.Contains(e.Id))
            .ToListAsync(cancellationToken);
    }

    /// <inheritdoc/>
    public async Task<List<Image>> GetFavoritesAsync(CancellationToken cancellationToken = default)
    {
        return await _context.Images
            .Where(e => e.IsFavorite)
            .OrderByDescending(e => e.ModifiedAt)
            .ToListAsync(cancellationToken);
    }

    /// <inheritdoc/>
    public async Task<List<Image>> GetNSFWAsync(CancellationToken cancellationToken = default)
    {
        return await _context.Images
            .Where(e => e.IsNSFW)
            .OrderByDescending(e => e.CreatedAt)
            .ToListAsync(cancellationToken);
    }

    /// <inheritdoc/>
    public async Task<List<Image>> SearchAsync(
        string? query,
        List<TagId>? tagIds,
        Rating? minRating,
        bool? favoritesOnly,
        bool? nsfwOnly,
        CancellationToken cancellationToken = default)
    {
        var images = _context.Images.AsQueryable();

        // Filter by favorites
        if (favoritesOnly == true)
        {
            images = images.Where(e => e.IsFavorite);
        }

        // Filter by NSFW
        if (nsfwOnly == true)
        {
            images = images.Where(e => e.IsNSFW);
        }

        // Filter by minimum rating
        if (minRating != null)
        {
            images = images.Where(e => e.Rating.Value >= minRating.Value);
        }

        // Filter by tags
        if (tagIds != null && tagIds.Count > 0)
        {
            var tagIdValues = tagIds.Select(t => (Guid)t).ToList();
            var imageIdsWithTags = await _context.Set<Dictionary<string, object>>("ImageTags")
                .Where(it => tagIdValues.Contains((Guid)it["TagId"]))
                .Select(it => (Guid)it["ImageId"])
                .Distinct()
                .ToListAsync(cancellationToken);

            images = images.Where(e => imageIdsWithTags.Contains(e.Id));
        }

        // Filter by query string (search in path)
        if (!string.IsNullOrWhiteSpace(query))
        {
            var queryLower = query.ToLowerInvariant();
            images = images.Where(e => e.Path.Value.ToLowerInvariant().Contains(queryLower));
        }

        return await images
            .OrderByDescending(e => e.CreatedAt)
            .ToListAsync(cancellationToken);
    }

    /// <inheritdoc/>
    public async Task AddAsync(Image image, CancellationToken cancellationToken = default)
    {
        await _context.Images.AddAsync(image, cancellationToken);
    }

    /// <inheritdoc/>
    public void Update(Image image)
    {
        _context.Images.Update(image);
    }

    /// <inheritdoc/>
    public void Delete(Image image)
    {
        _context.Images.Remove(image);
    }

    /// <inheritdoc/>
    public async Task<int> GetCountAsync(CancellationToken cancellationToken = default)
    {
        return await _context.Images.CountAsync(cancellationToken);
    }

    /// <inheritdoc/>
    public async Task<int> GetCountByFolderIdAsync(FolderId folderId, CancellationToken cancellationToken = default)
    {
        return await _context.Images
            .Where(e => e.FolderId == folderId)
            .CountAsync(cancellationToken);
    }

    /// <inheritdoc/>
    public async Task<bool> ExistsByHashAsync(Hash hash, CancellationToken cancellationToken = default)
    {
        return await _context.Images
            .AnyAsync(e => e.Hash == hash, cancellationToken);
    }

    /// <inheritdoc/>
    public async Task<bool> ExistsByPathAsync(FilePath path, CancellationToken cancellationToken = default)
    {
        return await _context.Images
            .AnyAsync(e => e.Path == path, cancellationToken);
    }
}