using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_Play3DSoundEvent
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.Play3DSoundEvent); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.Play3DSoundEvent)obj;
            //  Serialize EventId
            s.Write(value.EventId);
            //  Serialize Position
            s.Write(value.Position);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.Play3DSoundEvent)) as Rts.CnC.Messages.Client.Play3DSoundEvent;
            //  Deserialize EventId
            s.Read(out value.EventId);
            //  Deserialize Position
            s.Read(out value.Position);

            return value;
        }
        
    }
}
