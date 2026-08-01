using BerryAIGen.Domain.Common;
using BerryAIGen.Domain.Events;
using BerryAIGen.Domain.ValueObjects;

namespace BerryAIGen.Domain.Entities;

/// <summary>
/// Represents a user-created album for organizing images.
/// </summary>
public sealed class Album : AggregateRoot<AlbumId>
{
    private readonly List<ImageId> _imageIds = new();

    /// <summary>
    /// Gets the name of the album.
    /// </summary>
    public string Name { get; private set; }

    /// <summary>
    /// Gets the description of the album.
    /// </summary>
    public string? Description { get; private set; }

    /// <summary>
    /// Gets the cover image ID for this album.
    /// </summary>
    public ImageId? CoverImageId { get; private set; }

    /// <summary>
    /// Gets the creation timestamp.
    /// </summary>
    public DateTime CreatedAt { get; private set; }

    /// <summary>
    /// Gets the last modification timestamp.
    /// </summary>
    public DateTime ModifiedAt { get; private set; }

    /// <summary>
    /// Gets the read-only collection of image IDs in this album.
    /// </summary>
    public IReadOnlyCollection<ImageId> ImageIds => _imageIds.AsReadOnly();

    /// <summary>
    /// Gets the number of images in this album.
    /// </summary>
    public int ImageCount => _imageIds.Count;

    /// <summary>
    /// Private constructor for EF Core.
    /// </summary>
    private Album() { }

    /// <summary>
    /// Creates a new Album.
    /// </summary>
    /// <param name="name">The album name.</param>
    /// <param name="description">Optional description.</param>
    /// <returns>A new Album instance.</returns>
    public static Album Create(string name, string? description = null)
    {
        if (string.IsNullOrWhiteSpace(name))
            throw new ArgumentException("Album name cannot be null or whitespace.", nameof(name));

        var album = new Album
        {
            Id = AlbumId.New(),
            Name = name.Trim(),
            Description = description?.Trim(),
            CreatedAt = DateTime.UtcNow,
            ModifiedAt = DateTime.UtcNow
        };

        album.RaiseDomainEvent(new AlbumCreatedEvent(album.Id, name));
        return album;
    }

    /// <summary>
    /// Renames the album.
    /// </summary>
    /// <param name="newName">The new name.</param>
    public void Rename(string newName)
    {
        if (string.IsNullOrWhiteSpace(newName))
            throw new ArgumentException("Album name cannot be null or whitespace.", nameof(newName));

        Name = newName.Trim();
        ModifiedAt = DateTime.UtcNow;
    }

    /// <summary>
    /// Updates the album description.
    /// </summary>
    /// <param name="description">The new description.</param>
    public void UpdateDescription(string? description)
    {
        Description = description?.Trim();
        ModifiedAt = DateTime.UtcNow;
    }

    /// <summary>
    /// Sets the cover image for this album.
    /// </summary>
    /// <param name="imageId">The cover image ID.</param>
    public void SetCoverImage(ImageId? imageId)
    {
        CoverImageId = imageId;
        ModifiedAt = DateTime.UtcNow;
    }

    /// <summary>
    /// Adds an image to this album.
    /// </summary>
    /// <param name="imageId">The image ID to add.</param>
    public void AddImage(ImageId imageId)
    {
        if (imageId is null)
            throw new ArgumentNullException(nameof(imageId));

        if (!_imageIds.Contains(imageId))
        {
            _imageIds.Add(imageId);
            ModifiedAt = DateTime.UtcNow;
            RaiseDomainEvent(new ImageAddedToAlbumEvent(imageId, Id));
        }
    }

    /// <summary>
    /// Removes an image from this album.
    /// </summary>
    /// <param name="imageId">The image ID to remove.</param>
    public void RemoveImage(ImageId imageId)
    {
        if (_imageIds.Remove(imageId))
        {
            ModifiedAt = DateTime.UtcNow;
            RaiseDomainEvent(new ImageRemovedFromAlbumEvent(imageId, Id));
        }
    }

    /// <summary>
    /// Removes all images from this album.
    /// </summary>
    public void Clear()
    {
        _imageIds.Clear();
        ModifiedAt = DateTime.UtcNow;
    }
}