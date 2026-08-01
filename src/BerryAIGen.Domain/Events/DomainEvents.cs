using BerryAIGen.Domain.Common;
using BerryAIGen.Domain.ValueObjects;

namespace BerryAIGen.Domain.Events;

/// <summary>
/// Event raised when a new image is created.
/// </summary>
public sealed class ImageCreatedEvent : IDomainEvent
{
    /// <summary>
    /// Gets the image ID.
    /// </summary>
    public ImageId ImageId { get; }

    /// <summary>
    /// Gets the file path of the image.
    /// </summary>
    public FilePath Path { get; }

    /// <summary>
    /// Gets the hash of the image file.
    /// </summary>
    public Hash Hash { get; }

    /// <summary>
    /// Gets when this event occurred.
    /// </summary>
    public DateTime OccurredOn { get; }

    /// <summary>
    /// Initializes a new instance of ImageCreatedEvent.
    /// </summary>
    public ImageCreatedEvent(ImageId imageId, FilePath path, Hash hash)
    {
        ImageId = imageId ?? throw new ArgumentNullException(nameof(imageId));
        Path = path ?? throw new ArgumentNullException(nameof(path));
        Hash = hash ?? throw new ArgumentNullException(nameof(hash));
        OccurredOn = DateTime.UtcNow;
    }
}

/// <summary>
/// Event raised when an image is rated.
/// </summary>
public sealed class ImageRatedEvent : IDomainEvent
{
    public ImageId ImageId { get; }
    public Rating Rating { get; }
    public DateTime OccurredOn { get; }

    public ImageRatedEvent(ImageId imageId, Rating rating)
    {
        ImageId = imageId ?? throw new ArgumentNullException(nameof(imageId));
        Rating = rating ?? throw new ArgumentNullException(nameof(rating));
        OccurredOn = DateTime.UtcNow;
    }
}

/// <summary>
/// Event raised when an image is marked as favorite.
/// </summary>
public sealed class ImageMarkedFavoriteEvent : IDomainEvent
{
    public ImageId ImageId { get; }
    public DateTime OccurredOn { get; }

    public ImageMarkedFavoriteEvent(ImageId imageId)
    {
        ImageId = imageId ?? throw new ArgumentNullException(nameof(imageId));
        OccurredOn = DateTime.UtcNow;
    }
}

/// <summary>
/// Event raised when an image is removed from favorites.
/// </summary>
public sealed class ImageUnmarkedFavoriteEvent : IDomainEvent
{
    public ImageId ImageId { get; }
    public DateTime OccurredOn { get; }

    public ImageUnmarkedFavoriteEvent(ImageId imageId)
    {
        ImageId = imageId ?? throw new ArgumentNullException(nameof(imageId));
        OccurredOn = DateTime.UtcNow;
    }
}

/// <summary>
/// Event raised when an image is marked as NSFW.
/// </summary>
public sealed class ImageMarkedNSFWEvent : IDomainEvent
{
    public ImageId ImageId { get; }
    public DateTime OccurredOn { get; }

    public ImageMarkedNSFWEvent(ImageId imageId)
    {
        ImageId = imageId ?? throw new ArgumentNullException(nameof(imageId));
        OccurredOn = DateTime.UtcNow;
    }
}

/// <summary>
/// Event raised when an image NSFW flag is removed.
/// </summary>
public sealed class ImageUnmarkedNSFWEvent : IDomainEvent
{
    public ImageId ImageId { get; }
    public DateTime OccurredOn { get; }

    public ImageUnmarkedNSFWEvent(ImageId imageId)
    {
        ImageId = imageId ?? throw new ArgumentNullException(nameof(imageId));
        OccurredOn = DateTime.UtcNow;
    }
}

/// <summary>
/// Event raised when a tag is added to an image.
/// </summary>
public sealed class ImageTaggedEvent : IDomainEvent
{
    public ImageId ImageId { get; }
    public TagId TagId { get; }
    public DateTime OccurredOn { get; }

    public ImageTaggedEvent(ImageId imageId, TagId tagId)
    {
        ImageId = imageId ?? throw new ArgumentNullException(nameof(imageId));
        TagId = tagId ?? throw new ArgumentNullException(nameof(tagId));
        OccurredOn = DateTime.UtcNow;
    }
}

/// <summary>
/// Event raised when a tag is removed from an image.
/// </summary>
public sealed class ImageUntaggedEvent : IDomainEvent
{
    public ImageId ImageId { get; }
    public TagId TagId { get; }
    public DateTime OccurredOn { get; }

    public ImageUntaggedEvent(ImageId imageId, TagId tagId)
    {
        ImageId = imageId ?? throw new ArgumentNullException(nameof(imageId));
        TagId = tagId ?? throw new ArgumentNullException(nameof(tagId));
        OccurredOn = DateTime.UtcNow;
    }
}

/// <summary>
/// Event raised when an image is added to an album.
/// </summary>
public sealed class ImageAddedToAlbumEvent : IDomainEvent
{
    public ImageId ImageId { get; }
    public AlbumId AlbumId { get; }
    public DateTime OccurredOn { get; }

    public ImageAddedToAlbumEvent(ImageId imageId, AlbumId albumId)
    {
        ImageId = imageId ?? throw new ArgumentNullException(nameof(imageId));
        AlbumId = albumId ?? throw new ArgumentNullException(nameof(albumId));
        OccurredOn = DateTime.UtcNow;
    }
}

/// <summary>
/// Event raised when an image is removed from an album.
/// </summary>
public sealed class ImageRemovedFromAlbumEvent : IDomainEvent
{
    public ImageId ImageId { get; }
    public AlbumId AlbumId { get; }
    public DateTime OccurredOn { get; }

    public ImageRemovedFromAlbumEvent(ImageId imageId, AlbumId albumId)
    {
        ImageId = imageId ?? throw new ArgumentNullException(nameof(imageId));
        AlbumId = albumId ?? throw new ArgumentNullException(nameof(albumId));
        OccurredOn = DateTime.UtcNow;
    }
}

/// <summary>
/// Event raised when a new album is created.
/// </summary>
public sealed class AlbumCreatedEvent : IDomainEvent
{
    public AlbumId AlbumId { get; }
    public string Name { get; }
    public DateTime OccurredOn { get; }

    public AlbumCreatedEvent(AlbumId albumId, string name)
    {
        AlbumId = albumId ?? throw new ArgumentNullException(nameof(albumId));
        Name = name ?? throw new ArgumentNullException(nameof(name));
        OccurredOn = DateTime.UtcNow;
    }
}

/// <summary>
/// Event raised when a new folder is created.
/// </summary>
public sealed class FolderCreatedEvent : IDomainEvent
{
    public FolderId FolderId { get; }
    public FilePath Path { get; }
    public DateTime OccurredOn { get; }

    public FolderCreatedEvent(FolderId folderId, FilePath path)
    {
        FolderId = folderId ?? throw new ArgumentNullException(nameof(folderId));
        Path = path ?? throw new ArgumentNullException(nameof(path));
        OccurredOn = DateTime.UtcNow;
    }
}