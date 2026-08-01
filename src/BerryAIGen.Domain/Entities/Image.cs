using BerryAIGen.Domain.Common;
using BerryAIGen.Domain.Events;
using BerryAIGen.Domain.ValueObjects;

namespace BerryAIGen.Domain.Entities;

/// <summary>
/// Represents an image in the system.
/// This is an aggregate root that manages image metadata, tags, and albums.
/// </summary>
public sealed class Image : AggregateRoot<ImageId>
{
    private readonly List<TagId> _tagIds = new();
    private readonly List<AlbumId> _albumIds = new();

    /// <summary>
    /// Gets the file path where the image is stored.
    /// </summary>
    public FilePath Path { get; private set; }

    /// <summary>
    /// Gets the cryptographic hash of the image file.
    /// </summary>
    public Hash Hash { get; private set; }

    /// <summary>
    /// Gets the dimensions of the image.
    /// </summary>
    public ImageDimensions Dimensions { get; private set; }

    /// <summary>
    /// Gets the file size in bytes.
    /// </summary>
    public long FileSize { get; private set; }

    /// <summary>
    /// Gets the user-assigned rating (1-10, or 0 for unrated).
    /// </summary>
    public Rating Rating { get; private set; }

    /// <summary>
    /// Gets whether this image is marked as a favorite.
    /// </summary>
    public bool IsFavorite { get; private set; }

    /// <summary>
    /// Gets whether this image is flagged as NSFW (Not Safe For Work).
    /// </summary>
    public bool IsNSFW { get; private set; }

    /// <summary>
    /// Gets the aesthetic score (if calculated).
    /// </summary>
    public double? AestheticScore { get; private set; }

    /// <summary>
    /// Gets the folder ID where this image belongs.
    /// </summary>
    public FolderId FolderId { get; private set; }

    /// <summary>
    /// Gets the image type (e.g., "PNG", "JPG", "WebP").
    /// </summary>
    public string ImageType { get; private set; }

    /// <summary>
    /// Gets the creation timestamp.
    /// </summary>
    public DateTime CreatedAt { get; private set; }

    /// <summary>
    /// Gets the last modification timestamp.
    /// </summary>
    public DateTime ModifiedAt { get; private set; }

    /// <summary>
    /// Gets whether the image file is currently available on disk.
    /// </summary>
    public bool IsAvailable { get; private set; }

    /// <summary>
    /// Gets the read-only collection of tag IDs associated with this image.
    /// </summary>
    public IReadOnlyCollection<TagId> TagIds => _tagIds.AsReadOnly();

    /// <summary>
    /// Gets the read-only collection of album IDs containing this image.
    /// </summary>
    public IReadOnlyCollection<AlbumId> AlbumIds => _albumIds.AsReadOnly();

    /// <summary>
    /// Private constructor for EF Core.
    /// </summary>
    private Image() { }

    /// <summary>
    /// Creates a new Image entity.
    /// </summary>
    /// <param name="path">The file path of the image.</param>
    /// <param name="hash">The cryptographic hash of the file.</param>
    /// <param name="dimensions">The image dimensions.</param>
    /// <param name="fileSize">The file size in bytes.</param>
    /// <param name="folderId">The folder ID where the image belongs.</param>
    /// <param name="imageType">The image file type.</param>
    /// <returns>A new Image instance.</returns>
    public static Image Create(
        FilePath path,
        Hash hash,
        ImageDimensions dimensions,
        long fileSize,
        FolderId folderId,
        string imageType)
    {
        if (fileSize <= 0)
            throw new ArgumentException("File size must be greater than 0.", nameof(fileSize));

        if (string.IsNullOrWhiteSpace(imageType))
            throw new ArgumentException("Image type cannot be null or whitespace.", nameof(imageType));

        var image = new Image
        {
            Id = ImageId.New(),
            Path = path ?? throw new ArgumentNullException(nameof(path)),
            Hash = hash ?? throw new ArgumentNullException(nameof(hash)),
            Dimensions = dimensions ?? throw new ArgumentNullException(nameof(dimensions)),
            FileSize = fileSize,
            FolderId = folderId ?? throw new ArgumentNullException(nameof(folderId)),
            ImageType = imageType.ToUpperInvariant(),
            Rating = Rating.Default,
            IsFavorite = false,
            IsNSFW = false,
            IsAvailable = true,
            CreatedAt = DateTime.UtcNow,
            ModifiedAt = DateTime.UtcNow
        };

        image.RaiseDomainEvent(new ImageCreatedEvent(image.Id, path, hash));
        return image;
    }

    /// <summary>
    /// Sets the rating for this image.
    /// </summary>
    /// <param name="rating">The rating value (1-10).</param>
    public void SetRating(Rating rating)
    {
        Rating = rating ?? throw new ArgumentNullException(nameof(rating));
        ModifiedAt = DateTime.UtcNow;
        RaiseDomainEvent(new ImageRatedEvent(Id, rating));
    }

    /// <summary>
    /// Marks this image as a favorite.
    /// </summary>
    public void MarkAsFavorite()
    {
        if (!IsFavorite)
        {
            IsFavorite = true;
            ModifiedAt = DateTime.UtcNow;
            RaiseDomainEvent(new ImageMarkedFavoriteEvent(Id));
        }
    }

    /// <summary>
    /// Removes the favorite mark from this image.
    /// </summary>
    public void RemoveFromFavorites()
    {
        if (IsFavorite)
        {
            IsFavorite = false;
            ModifiedAt = DateTime.UtcNow;
            RaiseDomainEvent(new ImageUnmarkedFavoriteEvent(Id));
        }
    }

    /// <summary>
    /// Marks this image as NSFW.
    /// </summary>
    public void MarkAsNSFW()
    {
        if (!IsNSFW)
        {
            IsNSFW = true;
            ModifiedAt = DateTime.UtcNow;
            RaiseDomainEvent(new ImageMarkedNSFWEvent(Id));
        }
    }

    /// <summary>
    /// Removes the NSFW flag from this image.
    /// </summary>
    public void UnmarkNSFW()
    {
        if (IsNSFW)
        {
            IsNSFW = false;
            ModifiedAt = DateTime.UtcNow;
            RaiseDomainEvent(new ImageUnmarkedNSFWEvent(Id));
        }
    }

    /// <summary>
    /// Sets the aesthetic score for this image.
    /// </summary>
    /// <param name="score">The aesthetic score.</param>
    public void SetAestheticScore(double score)
    {
        if (score < 0 || score > 10)
            throw new ArgumentException("Aesthetic score must be between 0 and 10.", nameof(score));

        AestheticScore = score;
        ModifiedAt = DateTime.UtcNow;
    }

    /// <summary>
    /// Adds a tag to this image.
    /// </summary>
    /// <param name="tagId">The tag ID to add.</param>
    public void AddTag(TagId tagId)
    {
        if (tagId is null)
            throw new ArgumentNullException(nameof(tagId));

        if (!_tagIds.Contains(tagId))
        {
            _tagIds.Add(tagId);
            ModifiedAt = DateTime.UtcNow;
            RaiseDomainEvent(new ImageTaggedEvent(Id, tagId));
        }
    }

    /// <summary>
    /// Removes a tag from this image.
    /// </summary>
    /// <param name="tagId">The tag ID to remove.</param>
    public void RemoveTag(TagId tagId)
    {
        if (_tagIds.Remove(tagId))
        {
            ModifiedAt = DateTime.UtcNow;
            RaiseDomainEvent(new ImageUntaggedEvent(Id, tagId));
        }
    }

    /// <summary>
    /// Adds this image to an album.
    /// </summary>
    /// <param name="albumId">The album ID to add to.</param>
    public void AddToAlbum(AlbumId albumId)
    {
        if (albumId is null)
            throw new ArgumentNullException(nameof(albumId));

        if (!_albumIds.Contains(albumId))
        {
            _albumIds.Add(albumId);
            ModifiedAt = DateTime.UtcNow;
            RaiseDomainEvent(new ImageAddedToAlbumEvent(Id, albumId));
        }
    }

    /// <summary>
    /// Removes this image from an album.
    /// </summary>
    /// <param name="albumId">The album ID to remove from.</param>
    public void RemoveFromAlbum(AlbumId albumId)
    {
        if (_albumIds.Remove(albumId))
        {
            ModifiedAt = DateTime.UtcNow;
            RaiseDomainEvent(new ImageRemovedFromAlbumEvent(Id, albumId));
        }
    }

    /// <summary>
    /// Marks the image file as unavailable (missing or deleted).
    /// </summary>
    public void MarkAsUnavailable()
    {
        if (IsAvailable)
        {
            IsAvailable = false;
            ModifiedAt = DateTime.UtcNow;
        }
    }

    /// <summary>
    /// Marks the image file as available.
    /// </summary>
    public void MarkAsAvailable()
    {
        if (!IsAvailable)
        {
            IsAvailable = true;
            ModifiedAt = DateTime.UtcNow;
        }
    }

    /// <summary>
    /// Updates the image metadata.
    /// </summary>
    /// <param name="dimensions">New dimensions.</param>
    /// <param name="fileSize">New file size.</param>
    public void UpdateMetadata(ImageDimensions dimensions, long fileSize)
    {
        Dimensions = dimensions ?? throw new ArgumentNullException(nameof(dimensions));

        if (fileSize <= 0)
            throw new ArgumentException("File size must be greater than 0.", nameof(fileSize));

        FileSize = fileSize;
        ModifiedAt = DateTime.UtcNow;
    }
}