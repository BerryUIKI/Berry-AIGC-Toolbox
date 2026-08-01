namespace BerryAIGen.Domain.Common;

/// <summary>
/// Base class for aggregate roots in Domain-Driven Design.
/// An aggregate root is the only entry point to an aggregate and ensures consistency boundaries.
/// It can raise domain events that will be dispatched to event handlers.
/// </summary>
/// <typeparam name="TId">The type of the aggregate root's identifier.</typeparam>
public abstract class AggregateRoot<TId> : Entity<TId>
    where TId : notnull
{
    private readonly List<IDomainEvent> _domainEvents = new();

    /// <summary>
    /// Gets the domain events that have been raised by this aggregate root.
    /// </summary>
    public IReadOnlyCollection<IDomainEvent> DomainEvents => _domainEvents.AsReadOnly();

    /// <summary>
    /// Initializes a new instance of the <see cref="AggregateRoot{TId}"/> class.
    /// </summary>
    /// <param name="id">The unique identifier for this aggregate root.</param>
    protected AggregateRoot(TId id) : base(id)
    {
    }

    /// <summary>
    /// Parameterless constructor for ORM frameworks.
    /// </summary>
    protected AggregateRoot()
    {
    }

    /// <summary>
    /// Raises a domain event.
    /// Domain events represent something that happened in the domain that you want other parts of the same domain to be aware of.
    /// </summary>
    /// <param name="domainEvent">The domain event to raise.</param>
    protected void RaiseDomainEvent(IDomainEvent domainEvent)
    {
        _domainEvents.Add(domainEvent);
    }

    /// <summary>
    /// Clears all domain events.
    /// This should be called after the events have been dispatched.
    /// </summary>
    public void ClearDomainEvents()
    {
        _domainEvents.Clear();
    }
}