using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_EntityMoved
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.EntityMoved); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.EntityMoved)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize EntityId
            s.Write(value.EntityId);
            //  Serialize Position
            s.Write(value.Position);
            //  Serialize Facing
            s.Write(value.Facing);
            //  Serialize Velocity
            s.Write(value.Velocity);
            //  Serialize TurnDeltaAngle
            s.Write(value.TurnDeltaAngle);
            //  Serialize AngularVelocity
            s.Write(value.AngularVelocity);
            //  Serialize TimeStamp
            s.Write(value.TimeStamp);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.EntityMoved)) as Rts.CnC.Messages.Client.EntityMoved;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize EntityId
            s.Read(out value.EntityId);
            //  Deserialize Position
            s.Read(out value.Position);
            //  Deserialize Facing
            s.Read(out value.Facing);
            //  Deserialize Velocity
            s.Read(out value.Velocity);
            //  Deserialize TurnDeltaAngle
            s.Read(out value.TurnDeltaAngle);
            //  Deserialize AngularVelocity
            s.Read(out value.AngularVelocity);
            //  Deserialize TimeStamp
            s.Read(out value.TimeStamp);

            return value;
        }
        
    }
}
