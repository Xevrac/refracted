using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_Goodbye
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.Goodbye); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.Goodbye)obj;
            //  Serialize PlayerHandle
            s.Write(value.PlayerHandle);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.Goodbye)) as Rts.CnC.Messages.Client.Goodbye;
            //  Deserialize PlayerHandle
            s.Read(out value.PlayerHandle);

            return value;
        }
        
    }
}
